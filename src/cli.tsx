#!/usr/bin/env node
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Box, Text, render, useApp, useInput, useStdout } from 'ink'
import type { Agent } from '@mastra/core/agent'
import {
  alice,
  bob,
  carol,
  createCoworkerAgent,
  subscribeCoworkerThread,
  subscribeCoworkerThreads,
  unsubscribeCoworkerThread,
} from './agents/coworkers.js'
import { describeModelConfiguration } from './agents/model.js'
import { addAgentCoworker, agentCoworkerProfiles, removeAgentCoworker } from './office/coworkers.js'
import { deliverHumanMessage, listHumanInbox, markAllHumanMessagesSeen } from './office/human.js'
import { officeMailbox } from './office/mailbox.js'
import { coworkerThread } from './office/threads.js'
import { mastra } from './mastra/index.js'
import { listCoworkerStatuses, listOfficeTasks, type CoworkerStatus, type OfficeTask } from './office/kanban.js'
import { answerHumanQuestion, listHumanQuestions, type HumanQuestion } from './office/human-questions.js'
import { expandLogLines, logContentWidth, type LogLine } from './cli/log-text.js'
import { formatOfficeEvent, handleReasoningChunk, type Chunk } from './cli/office-events.js'
import { ChatPanel, CoworkerPanel, QuestionPanel } from './cli/panels.js'

type Mode = 'chat' | 'coworkers' | 'questions' | 'add-name' | 'add-role'

let nextLogId = 1

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
  const threadSubscriptions = useRef(new Map<string, { unsubscribe: () => void }>())
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

  const addLog = useCallback((text: string, color?: string) => {
    setLogs(current => [...current.slice(-1000), { id: nextLogId++, text, color }])
  }, [])

  const watchAgent = useCallback(async (agentName: string, agent: Agent) => {
    const lowerName = agentName.toLowerCase()
    if (watching.current.has(lowerName)) return
    watching.current.add(lowerName)
    try {
      const subscription = await agent.subscribeToThread(coworkerThread(agentName))
      threadSubscriptions.current.set(lowerName, subscription)
      for await (const chunk of subscription.stream) {
        const typed = chunk as Chunk
        try {
          if (handleReasoningChunk(agentName, typed, addLog)) continue
          const line = formatOfficeEvent(agentName, typed)
          if (line) addLog(line.text, line.color)
        } catch (error) {
          addLog(`⚠️ Could not render an event from ${agentName}: ${error instanceof Error ? error.message : String(error)}`, 'red')
        }
      }
    } catch (error) {
      addLog(`❌ Stopped watching ${agentName}: ${error instanceof Error ? error.message : String(error)}`, 'red')
    } finally {
      threadSubscriptions.current.delete(lowerName)
      watching.current.delete(lowerName)
    }
  }, [addLog])

  function unwatchAgent(agentName: string) {
    const lowerName = agentName.toLowerCase()
    threadSubscriptions.current.get(lowerName)?.unsubscribe()
    threadSubscriptions.current.delete(lowerName)
    watching.current.delete(lowerName)
  }

  useEffect(() => {
    for (const name of Object.keys(agents)) void watchAgent(name, agents[name]!)
  }, [agents, watchAgent])

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset the selection state whenever the active question changes
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
  }, [addLog])

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

    // Only treat "?" as a hotkey where it cannot be part of typed text: an
    // empty chat input or the coworkers panel.
    if (input === '?' && questions.length > 0 && ((mode === 'chat' && message === '') || mode === 'coworkers')) {
      setMode('questions')
      return
    }

    if ((key as any).pageUp) return setLogScrollOffset(offset => Math.min(Math.max(0, displayLines.length - visibleLogCount), offset + visibleLogCount))
    if ((key as any).pageDown) return setLogScrollOffset(offset => Math.max(0, offset - visibleLogCount))
    if ((key as any).end) return setLogScrollOffset(0)
    if ((key as any).home) return setLogScrollOffset(Math.max(0, displayLines.length - visibleLogCount))

    if (mode === 'questions') {
      const question = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
      if (key.escape) {
        if (customSelected) {
          setCustomSelected(false)
          setCustomAnswer('')
          return
        }
        return setMode('chat')
      }
      if (key.upArrow) return setQuestionIndex(index => Math.max(0, index - 1))
      if (key.downArrow) return setQuestionIndex(index => Math.min(Math.max(0, questions.length - 1), index + 1))
      if (!question) return
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
      // While the custom answer is active, all printable input (digits included)
      // is text; Esc deselects it. Digit hotkeys only apply otherwise.
      if (customSelected) {
        if (key.backspace || key.delete) return setCustomAnswer(value => value.slice(0, -1))
        if (input && !key.ctrl && !key.meta) setCustomAnswer(value => value + input)
        return
      }
      if (/^[0-9]$/.test(input)) {
        const optionIndex = (input === '0' ? 10 : Number(input)) - 1
        const customIndex = question.choices.length
        if (question.allowCustomAnswer && optionIndex === customIndex) {
          setCustomSelected(true)
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
          unsubscribeCoworkerThread(removed.name)
          unwatchAgent(removed.name)
          setAgents(current => {
            const { [removed.name]: _removed, ...rest } = current
            return rest
          })
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
subscribeCoworkerThreads()
render(<App />)
