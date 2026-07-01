#!/usr/bin/env node
import React, { useEffect, useMemo, useRef, useState } from 'react'
import { Box, Text, render, useApp, useInput, useStdout } from 'ink'
import stringWidth from 'string-width'
import { Agent } from '@mastra/core/agent'
import {
  alice,
  bob,
  carol,
  createCoworkerAgent,
  subscribeCoworkerThread,
  subscribeCoworkerThreads,
} from './agents/coworkers.js'
import { describeModelConfiguration } from './agents/model.js'
import { addAgentCoworker, agentCoworkerProfiles, removeAgentCoworker } from './office/coworkers.js'
import { deliverHumanMessage, listHumanInbox, markAllHumanMessagesSeen } from './office/human.js'
import { officeMailbox } from './office/mailbox.js'
import { coworkerThread } from './office/threads.js'
import { mastra } from './mastra/index.js'
import { listCoworkerStatuses, listOfficeTasks, type CoworkerStatus, type OfficeTask } from './office/kanban.js'
import { answerHumanQuestion, listHumanQuestions, type HumanQuestion } from './office/human-questions.js'

type AgentName = string
type LogLine = { id: number; text: string; color?: string }
type DisplayLine = { key: string; text: string; color?: string }
type Chunk = { type?: string; payload?: any; data?: any }
type Mode = 'chat' | 'coworkers' | 'questions' | 'add-name' | 'add-role'

subscribeCoworkerThreads()

let nextLogId = 1
const reasoningBuffers = new Map<string, string>()
const activeReasoning = new Set<string>()

function normalizeLogText(text: string) {
  return text.replace(/\s+/g, ' ').trim()
}

function logContentWidth(columns: number) {
  return Math.max(20, columns - 12)
}

function wrapLogText(text: string, width: number): string[] {
  if (width < 1) return [text]

  const lines: string[] = []
  for (const paragraph of text.replace(/\r\n/g, '\n').split('\n')) {
    if (!paragraph) {
      lines.push('')
      continue
    }

    let current = ''
    let currentWidth = 0

    const pushLine = () => {
      if (!current) return
      lines.push(current.trimEnd())
      current = ''
      currentWidth = 0
    }

    const pushChars = (value: string) => {
      for (const char of value) {
        const charWidth = stringWidth(char)
        if (currentWidth + charWidth > width && currentWidth > 0) {
          pushLine()
        }
        current += char
        currentWidth += charWidth
      }
    }

    for (const word of paragraph.split(/(\s+)/)) {
      if (!word) continue
      const wordWidth = stringWidth(word)

      if (wordWidth > width) {
        pushLine()
        pushChars(word)
        continue
      }

      if (currentWidth + wordWidth > width && currentWidth > 0) {
        pushLine()
      }

      current += word
      currentWidth += wordWidth
    }

    pushLine()
  }

  return lines.length > 0 ? lines : ['']
}

function expandLogLines(logs: LogLine[], width: number): DisplayLine[] {
  const expanded: DisplayLine[] = []
  for (const log of logs) {
    const parts = wrapLogText(log.text, width)
    parts.forEach((text, index) => {
      expanded.push({ key: `${log.id}:${index}`, text, color: log.color })
    })
  }
  return expanded
}

function handleReasoningChunk(agentName: string, chunk: Chunk, addLog: (text: string, color?: string) => void) {
  switch (chunk.type) {
    case 'reasoning-start': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      reasoningBuffers.set(key, '')
      if (activeReasoning.has(key)) return true
      activeReasoning.add(key)
      addLog(`💭 ${agentName} is thinking…`, 'gray')
      return true
    }
    case 'reasoning-delta': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      const delta = chunk.payload?.text ?? ''
      reasoningBuffers.set(key, `${reasoningBuffers.get(key) ?? ''}${delta}`)
      return true
    }
    case 'reasoning-end': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      activeReasoning.delete(key)
      const reasoning = normalizeLogText(reasoningBuffers.get(key) ?? '')
      reasoningBuffers.delete(key)
      if (reasoning) addLog(`💭 ${agentName} reasoning: ${reasoning}`, 'gray')
      return true
    }
    default:
      return false
  }
}

function App() {
  const { exit } = useApp()
  const { stdout } = useStdout()
  const [mode, setMode] = useState<Mode>('chat')
  const [agents, setAgents] = useState<Record<string, Agent>>({ Alice: alice, Bob: bob, Carol: carol })
  const [names, setNames] = useState<string[]>(['All', ...agentCoworkerProfiles.map(profile => profile.name)])
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [message, setMessage] = useState('')
  const [draftName, setDraftName] = useState('')
  const [draftRole, setDraftRole] = useState('')
  const [logScrollOffset, setLogScrollOffset] = useState(0)
  const [statuses, setStatuses] = useState<CoworkerStatus[]>([])
  const [tasks, setTasks] = useState<OfficeTask[]>([])
  const [questions, setQuestions] = useState<HumanQuestion[]>([])
  const [questionIndex, setQuestionIndex] = useState(0)
  const [selectedChoices, setSelectedChoices] = useState<string[]>([])
  const [customSelected, setCustomSelected] = useState(false)
  const [customAnswer, setCustomAnswer] = useState('')
  const watching = useRef(new Set<string>())
  const deliveryQueue = useRef(Promise.resolve())
  const [logs, setLogs] = useState<LogLine[]>([
    { id: nextLogId++, text: 'Agent office ready. Tab switches Chat/Coworkers. Enter sends.', color: 'green' },
  ])

  const terminalRows = stdout.rows || 40
  const terminalColumns = stdout.columns || 80
  const inputPanelHeight = mode === 'chat' ? 7 : mode === 'questions' ? 14 : 9
  const logPanelHeight = Math.max(8, terminalRows - inputPanelHeight)
  const visibleLogCount = Math.max(1, logPanelHeight - 3)
  const logWrapWidth = logContentWidth(terminalColumns)
  const displayLines = useMemo(() => expandLogLines(logs, logWrapWidth), [logs, logWrapWidth])
  const selectedName = names[Math.min(selectedIndex, Math.max(0, names.length - 1))]

  const addLog = (text: string, color?: string) => {
    setLogs(current => [...current.slice(-1000), { id: nextLogId++, text, color }])
  }

  async function watchAgent(agentName: string, agent: Agent) {
    if (watching.current.has(agentName.toLowerCase())) return
    watching.current.add(agentName.toLowerCase())
    const subscription = await agent.subscribeToThread(coworkerThread(agentName))
    for await (const chunk of subscription.stream) {
      const typed = chunk as Chunk
      if (handleReasoningChunk(agentName, typed, addLog)) continue
      const line = formatOfficeEvent(agentName, typed)
      if (line) addLog(line.text, line.color)
    }
  }

  useEffect(() => {
    for (const name of Object.keys(agents)) void watchAgent(name, agents[name]!)
  }, [agents])

  useEffect(() => {
    setSelectedChoices([])
    setCustomSelected(false)
    setCustomAnswer('')
  }, [questionIndex])

  useEffect(() => {
    const officePoll = setInterval(() => {
      setStatuses(listCoworkerStatuses())
      setTasks(listOfficeTasks())
      setQuestions(listHumanQuestions({ unansweredOnly: true }))
      const unread = listHumanInbox({ unreadOnly: true })
      if (unread.length === 0) return
      for (const inboxMessage of unread) addLog(`📬 Human inbox — ${inboxMessage.from}: ${inboxMessage.body}`, 'green')
      markAllHumanMessagesSeen()
    }, 1_000)
    return () => clearInterval(officePoll)
  }, [])

  useInput((input, key) => {
    if (key.ctrl && input === 'c') exit()

    if (key.escape) {
      if (mode === 'coworkers') {
        setMode('chat')
        return
      }
    }

    if (key.tab) {
      setMode(current => (current === 'chat' ? 'coworkers' : current === 'coworkers' && questions.length > 0 ? 'questions' : 'chat'))
      setMessage('')
      return
    }

    if (input === '?' && questions.length > 0 && mode !== 'add-name' && mode !== 'add-role') {
      setMode('questions')
      return
    }

    if ((key as any).pageUp) return setLogScrollOffset(offset => Math.min(Math.max(0, displayLines.length - visibleLogCount), offset + visibleLogCount))
    if ((key as any).pageDown) return setLogScrollOffset(offset => Math.max(0, offset - visibleLogCount))
    if ((key as any).end) return setLogScrollOffset(0)
    if ((key as any).home) return setLogScrollOffset(Math.max(0, displayLines.length - visibleLogCount))

    if (mode === 'questions') {
      const question = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
      if (key.escape) return setMode('chat')
      if (key.upArrow) return setQuestionIndex(index => Math.max(0, index - 1))
      if (key.downArrow) return setQuestionIndex(index => Math.min(Math.max(0, questions.length - 1), index + 1))
      if (!question) return
      if (/^[0-9]$/.test(input)) {
        const optionIndex = (input === '0' ? 10 : Number(input)) - 1
        const customIndex = question.choices.length
        if (question.allowCustomAnswer && optionIndex === customIndex) {
          setCustomSelected(value => {
            if (value) setCustomAnswer('')
            return !value
          })
          if (question.mode === 'single') setSelectedChoices([])
          return
        }
        const choice = question.choices[optionIndex]
        if (!choice) return
        if (question.mode === 'single') {
          setSelectedChoices([choice])
          setCustomSelected(false)
          setCustomAnswer('')
        } else setSelectedChoices(current => current.includes(choice) ? current.filter(item => item !== choice) : [...current, choice])
        return
      }
      if (key.return) {
        try {
          answerHumanQuestion({ id: question.id, selectedChoices, customAnswer: customSelected ? customAnswer : undefined })
          addLog(`✅ Human answered ${question.from}'s question`, 'green')
          setSelectedChoices([])
          setCustomSelected(false)
          setCustomAnswer('')
          setQuestions(listHumanQuestions({ unansweredOnly: true }))
          if (questions.length <= 1) setMode('chat')
        } catch (error) {
          addLog(`❌ Could not answer question: ${error instanceof Error ? error.message : String(error)}`, 'red')
        }
        return
      }
      if (!customSelected) return
      if (key.backspace || key.delete) return setCustomAnswer(value => value.slice(0, -1))
      if (input && !key.ctrl && !key.meta) setCustomAnswer(value => value + input)
      return
    }

    if (mode === 'add-name' || mode === 'add-role') {
      const setter = mode === 'add-name' ? setDraftName : setDraftRole
      if (key.escape) {
        setMode('coworkers')
        setDraftName('')
        setDraftRole('')
        return
      }
      if (key.return) {
        if (mode === 'add-name') setMode('add-role')
        else addCoworker()
        return
      }
      if (key.backspace || key.delete) return setter(value => value.slice(0, -1))
      if (input && !key.ctrl && !key.meta) setter(value => value + input)
      return
    }

    if (key.upArrow) return setSelectedIndex(index => (index - 1 + names.length) % names.length)
    if (key.downArrow) return setSelectedIndex(index => (index + 1) % names.length)

    if (mode === 'coworkers') {
      if (input === 'a') {
        setDraftName('')
        setDraftRole('')
        setMode('add-name')
        return
      }
      if (input === 'd' && selectedName && selectedName.toLowerCase() !== 'all') {
        const removed = removeAgentCoworker(selectedName)
        if (removed) {
          setNames(current => current.filter(name => name !== selectedName))
          addLog(`🗑️ Removed coworker ${selectedName} for this run`, 'red')
          setSelectedIndex(0)
        }
        return
      }
      return
    }

    if (key.return) {
      const body = message.trim()
      if (!body || !selectedName) return
      setMessage('')
      const recipient = selectedName
      deliveryQueue.current = deliveryQueue.current
        .then(() => deliverHumanMessage(recipient, body))
        .then(sent => addLog(`🧑 Human → ${recipient}: ${sent.body}`, 'cyan'))
        .catch(error => addLog(`❌ Could not deliver to ${recipient}: ${error instanceof Error ? error.message : String(error)}`, 'red'))
      return
    }
    if (key.backspace || key.delete) return setMessage(value => value.slice(0, -1))
    if (input && !key.ctrl && !key.meta) setMessage(value => value + input)
  })

  function addCoworker() {
    try {
      const profile = addAgentCoworker({ name: draftName, role: draftRole })
      const agent = createCoworkerAgent(profile.name, profile.role)
      mastra.addAgent(agent, profile.name.toLowerCase())
      subscribeCoworkerThread(profile.name)
      setAgents(current => ({ ...current, [profile.name]: agent }))
      setNames(current => [...current, profile.name])
      setSelectedIndex(names.length)
      for (const name of names.filter(name => name.toLowerCase() !== 'all')) {
        officeMailbox.send({
          from: 'office-admin',
          to: name,
          body: `Coworker directory updated: ${profile.name} joined as ${profile.role}. Use list_coworkers for the current full list.`,
        })
      }
      addLog(`➕ Added coworker ${profile.name}: ${profile.role}`, 'green')
      setDraftName('')
      setDraftRole('')
      setMode('coworkers')
    } catch (error) {
      addLog(`❌ Could not add coworker: ${error instanceof Error ? error.message : String(error)}`, 'red')
      setMode('coworkers')
    }
  }

  const maxScrollOffset = Math.max(0, displayLines.length - visibleLogCount)
  const effectiveScrollOffset = Math.min(logScrollOffset, maxScrollOffset)
  const logEnd = displayLines.length - effectiveScrollOffset
  const logStart = Math.max(0, logEnd - visibleLogCount)
  const visibleLogs = useMemo(() => displayLines.slice(logStart, logEnd), [displayLines, logStart, logEnd])

  return (
    <Box flexDirection="column" height={terminalRows} paddingX={1}>
      <Box borderStyle="round" borderColor="gray" flexDirection="column" height={logPanelHeight} paddingX={1}>
        <Box justifyContent="space-between">
          <Text bold>Office log</Text>
          <Text color="gray">{effectiveScrollOffset > 0 ? `scroll ${effectiveScrollOffset} · ` : ''}PgUp/PgDn · End latest</Text>
        </Box>
        <Text>
          {visibleLogs.map((line, index) => (
            <Text key={line.key} color={line.color as any}>
              {line.text}
              {index < visibleLogs.length - 1 ? '\n' : ''}
            </Text>
          ))}
        </Text>
      </Box>

      <Box borderStyle="round" borderColor={mode === 'chat' ? 'cyan' : mode === 'questions' ? 'yellow' : 'green'} paddingX={1} flexDirection="column" height={inputPanelHeight}>
        <Text bold>{mode === 'chat' ? 'Send message' : mode === 'questions' ? 'Human questions' : 'Manage coworkers'}</Text>
        {mode === 'chat' && <ChatPanel names={names} selectedIndex={selectedIndex} selectedName={selectedName} message={message} statuses={statuses} tasks={tasks} questionCount={questions.length} />}
        {mode === 'coworkers' && <CoworkerPanel names={names} selectedIndex={selectedIndex} statuses={statuses} tasks={tasks} />}
        {mode === 'questions' && <QuestionPanel questions={questions} questionIndex={questionIndex} selectedChoices={selectedChoices} customSelected={customSelected} customAnswer={customAnswer} />}
        {mode === 'add-name' && <Text>New coworker name: {draftName}<Text color="gray">█</Text></Text>}
        {mode === 'add-role' && <Text>Role for {draftName}: {draftRole}<Text color="gray">█</Text></Text>}
      </Box>
    </Box>
  )
}

function statusFor(statuses: CoworkerStatus[], name: string) {
  return statuses.find(status => status.name.toLowerCase() === name.toLowerCase())
}

function kanbanSummary(tasks: OfficeTask[]) {
  const columns = ['todo', 'doing', 'blocked', 'review', 'done']
  return columns.map(column => `${column}:${tasks.filter(task => task.status === column).length}`).join(' ')
}

function ChatPanel({ names, selectedIndex, selectedName, message, statuses, tasks, questionCount }: { names: string[]; selectedIndex: number; selectedName?: string; message: string; statuses: CoworkerStatus[]; tasks: OfficeTask[]; questionCount: number }) {
  return <>
    <Text>Tab manage coworkers{questionCount > 0 ? ` · ? answer ${questionCount} question${questionCount === 1 ? '' : 's'}` : ''} · ↑/↓ select · Enter send · Ctrl+C quit</Text>
    <Box marginTop={1}>{names.map((name, index) => {
      const status = statusFor(statuses, name)
      const label = `${name}${status ? `:${status.status}` : ''}`
      return <Box key={name} marginRight={2}><Text color={index === selectedIndex ? 'cyan' : undefined} inverse={index === selectedIndex}>{index === selectedIndex ? ` ${label} ` : label}</Text></Box>
    })}</Box>
    <Text color="gray">Kanban {kanbanSummary(tasks)}</Text>
    <Box><Text color="cyan">To {selectedName}: </Text><Text>{message}</Text><Text color="gray">█</Text></Box>
  </>
}

function QuestionPanel({ questions, questionIndex, selectedChoices, customSelected, customAnswer }: { questions: HumanQuestion[]; questionIndex: number; selectedChoices: string[]; customSelected: boolean; customAnswer: string }) {
  const question = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
  if (!question) return <Text color="gray">No pending questions. Esc/Tab returns to chat.</Text>
  return <>
    <Text>Esc chat · ↑/↓ question · number toggles/selects · select Custom to type · Enter submit</Text>
    <Text color="yellow">{questionIndex + 1}/{questions.length} From {question.from}: {question.prompt}</Text>
    <Text color="gray">Mode: {question.mode} · custom answer allowed</Text>
    {question.choices.slice(0, 9).map((choice, index) => (
      <Text key={choice} color={selectedChoices.includes(choice) ? 'green' : undefined}>{index + 1}. {selectedChoices.includes(choice) ? '☑' : '☐'} {choice}</Text>
    ))}
    {question.allowCustomAnswer && <Text color={customSelected ? 'green' : undefined}>{question.choices.length + 1}. {customSelected ? '☑' : '☐'} Custom{customSelected ? `: ${customAnswer}` : ''}{customSelected ? <Text color="gray">█</Text> : ''}</Text>}
  </>
}

function CoworkerPanel({ names, selectedIndex, statuses, tasks }: { names: string[]; selectedIndex: number; statuses: CoworkerStatus[]; tasks: OfficeTask[] }) {
  const activeTasks = tasks.filter(task => task.status !== 'done' && task.status !== 'canceled').slice(0, 3)
  return <>
    <Text>Esc/Tab chat · ↑/↓ select · a add runtime coworker · d remove for this run · Ctrl+C quit</Text>
    {names.map((name, index) => {
      const status = statusFor(statuses, name)
      return <Text key={name} color={index === selectedIndex ? 'green' : undefined} inverse={index === selectedIndex}>{index === selectedIndex ? ` ${name} ` : ` ${name}`}<Text color="gray"> {status ? `${status.status}${status.note ? ` — ${status.note}` : ''}` : 'available'}</Text></Text>
    })}
    <Text color="gray">Kanban {kanbanSummary(tasks)}</Text>
    {activeTasks.map(task => <Text key={task.id}>• [{task.status}] {task.title}{task.assignee ? ` @${task.assignee}` : ''}</Text>)}
  </>
}

function formatOfficeEvent(agentName: AgentName, chunk: Chunk): { text: string; color?: string } | undefined {
  switch (chunk.type) {
    case 'data-signal': {
      const attrs = chunk.data?.attributes ?? {}
      const contents = chunk.data?.contents
      if (attrs.kind === 'direct-message' && contents) return { text: `📨 ${agentName} received: ${contents}`, color: 'yellow' }
      return
    }
    case 'tool-call': {
      const { toolName, args } = chunk.payload ?? {}
      if (toolName === 'send_office_message') return { text: `${args.to?.toLowerCase?.() === 'all' ? '📣' : '✉️ '} ${agentName} → ${args.to}: ${args.message}`, color: 'magenta' }
      if (toolName === 'ask_human_question') return { text: `❓ ${agentName} asked Human: ${args.prompt}`, color: 'yellow' }
      if (toolName === 'set_status') return { text: `📍 ${agentName} status → ${args.status}${args.note ? `: ${args.note}` : ''}`, color: 'blue' }
      if (toolName === 'create_office_task') return { text: `📋 ${agentName} created task: ${args.title}`, color: 'green' }
      if (toolName === 'update_office_task') return { text: `📋 ${agentName} updated task ${args.id}${args.status ? ` → ${args.status}` : ''}`, color: 'green' }
      if (toolName === 'list_office_tasks') return { text: `📋 ${agentName} checked the Kanban board`, color: 'gray' }
      if (toolName === 'list_statuses') return { text: `📍 ${agentName} checked coworker statuses`, color: 'gray' }
      if (toolName === 'list_coworkers') return { text: `👥 ${agentName} checked the coworker directory`, color: 'gray' }
      if (toolName === 'get_current_time') return { text: `🕒 ${agentName} checked current Unix time`, color: 'blue' }
      if (toolName === 'convert_unix_time_to_iso8601') return { text: `🕒 ${agentName} converted Unix time ${args.unixTime} (${args.unit ?? 'seconds'}) to ISO 8601`, color: 'blue' }
      return { text: `🛠️  ${agentName} called ${toolName}${args ? ` ${JSON.stringify(args)}` : ''}`, color: 'gray' }
    }
    case 'tool-result': {
      const { toolName, result } = chunk.payload ?? {}
      if (toolName === 'ask_human_question') return { text: `❓ ${agentName} queued question ${result.id}`, color: 'yellow' }
      if (toolName === 'set_status') return { text: `📍 ${agentName} status set: ${result.status}${result.note ? ` — ${result.note}` : ''}`, color: 'blue' }
      if (toolName === 'create_office_task') return { text: `📋 Created task ${result.id}: ${result.title}`, color: 'green' }
      if (toolName === 'update_office_task') return { text: `📋 Task ${result.id}: ${result.status} — ${result.title}`, color: 'green' }
      if (toolName === 'list_office_tasks') return { text: `📋 Kanban has ${result.tasks?.length ?? 0} task(s)`, color: 'gray' }
      if (toolName === 'list_statuses') return { text: `📍 Found ${result.statuses?.length ?? 0} coworker status(es)`, color: 'gray' }
      if (toolName === 'list_coworkers') return { text: `👥 Found ${result.coworkers?.length ?? 0} coworker(s)`, color: 'gray' }
      if (toolName === 'list_office_messages') return { text: `✉️  Mailbox has ${result.messages?.length ?? 0} message(s)`, color: 'gray' }
      if (toolName === 'schedule_self_wake') return { text: `⏰ ${agentName} scheduled wake in ${result.delaySeconds}s: ${result.instruction}`, color: 'blue' }
      if (toolName === 'clear_scheduled_action' && result?.cleared) return { text: `🧹 ${agentName} cleared scheduled wake`, color: 'blue' }
      if (toolName === 'get_current_time') return { text: `🕒 ${agentName} current Unix time: ${result.unixTime}`, color: 'blue' }
      if (toolName === 'convert_unix_time_to_iso8601') return { text: `🕒 ${agentName} ISO 8601 time: ${result.iso8601}`, color: 'blue' }
      return { text: `✅ ${agentName} ${toolName} result${result ? ` ${JSON.stringify(result).slice(0, 500)}` : ''}`, color: 'gray' }
    }
    case 'error': {
      const error = chunk.payload?.error
      return { text: `❌ ${agentName} error: ${error?.responseBody ?? error?.message ?? 'unknown error'}`, color: 'red' }
    }
  }
}

function ensureModelProviderConfigured() {
  const { hasXaiKey, hasAnthropicKey, hasOpenaiKey } = describeModelConfiguration()
  if (hasXaiKey || hasAnthropicKey || hasOpenaiKey) return

  console.error(`agent-office needs a model provider API key.

Set one of:
  export XAI_API_KEY=...
  export ANTHROPIC_API_KEY=...
  export OPENAI_API_KEY=...

Optional override:
  export MASTRA_MODEL=xai/grok-4.3
`)
  process.exit(1)
}

ensureModelProviderConfigured()
render(<App />)