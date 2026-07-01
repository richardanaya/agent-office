#!/usr/bin/env node
import { existsSync } from 'node:fs'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Box, Text, render, useApp, useInput, useStdout } from 'ink'
import type { Agent } from '@mastra/core/agent'
import { getCoworkerAgent } from './agents/coworkers.js'
import { describeModelConfiguration } from './agents/model.js'
import { listAgentCoworkers } from './office/coworkers.js'
import { fireCoworker, hireCoworker } from './office/hiring.js'
import { loadTeamFile, saveTeamFile, type TeamFile } from './office/team.js'
import { deliverHumanMessage, listHumanInbox, markAllHumanMessagesSeen } from './office/human.js'
import { officeMailbox } from './office/mailbox.js'
import { coworkerThread } from './office/threads.js'
import { listCoworkerStatuses, listOfficeTasks, type CoworkerStatus, type OfficeTask } from './office/kanban.js'
import { answerHumanQuestion, listHumanQuestions, type HumanQuestion } from './office/human-questions.js'
import { expandLogLines, logContentWidth, type LogLine } from './cli/log-text.js'
import { formatOfficeEvent, handleReasoningChunk, type Chunk } from './cli/office-events.js'
import { parseMessageTarget } from './cli/message-target.js'
import { ChatPanel, QuestionPanel, TeamPanel } from './cli/panels.js'

type Mode = 'chat' | 'team' | 'questions' | 'add-name' | 'add-role' | 'save-team' | 'load-team'

const DEFAULT_TEAM_FILE = 'team.json'
const MESSAGE_HISTORY_LIMIT = 50

let nextLogId = 1

function App({ initialTeamFile, initialLogs }: { initialTeamFile: string; initialLogs: { text: string; color?: string }[] }) {
  const { exit } = useApp()
  const { stdout } = useStdout()
  const [mode, setMode] = useState<Mode>(() => (listAgentCoworkers().length === 0 ? 'team' : 'chat'))
  const [agents, setAgents] = useState<Record<string, Agent>>(() =>
    Object.fromEntries(listAgentCoworkers().flatMap(profile => {
      const agent = getCoworkerAgent(profile.name)
      return agent ? [[profile.name, agent] as const] : []
    })),
  )
  const [names, setNames] = useState<string[]>(() => ['All', ...listAgentCoworkers().map(profile => profile.name)])
  const [pinnedIndex, setPinnedIndex] = useState(0)
  const [teamIndex, setTeamIndex] = useState(0)
  const [message, setMessage] = useState('')
  const [history, setHistory] = useState<string[]>([])
  const [historyIndex, setHistoryIndex] = useState<number | null>(null)
  const [draftName, setDraftName] = useState('')
  const [draftRole, setDraftRole] = useState('')
  const [teamFile, setTeamFile] = useState(initialTeamFile)
  const [draftFile, setDraftFile] = useState('')
  const [logScrollOffset, setLogScrollOffset] = useState(0)
  const [statuses, setStatuses] = useState<CoworkerStatus[]>([])
  const [tasks, setTasks] = useState<OfficeTask[]>([])
  const [questions, setQuestions] = useState<HumanQuestion[]>([])
  const [questionIndex, setQuestionIndex] = useState(0)
  const [highlightIndex, setHighlightIndex] = useState(0)
  const [selectedChoices, setSelectedChoices] = useState<string[]>([])
  const [customAnswer, setCustomAnswer] = useState('')
  const watching = useRef(new Set<string>())
  const threadSubscriptions = useRef(new Map<string, { unsubscribe: () => void }>())
  const deliveryQueue = useRef(Promise.resolve())
  const [logs, setLogs] = useState<LogLine[]>(() => [
    { id: nextLogId++, text: 'Welcome to your agent office. Tab switches Chat/Team.', color: 'green' },
    ...initialLogs.map(line => ({ id: nextLogId++, ...line })),
  ])

  const coworkerNames = useMemo(() => names.filter(name => name !== 'All'), [names])
  const activeQuestion = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
  const questionChoices = activeQuestion ? activeQuestion.choices.slice(0, 9) : []
  const questionRows = activeQuestion ? questionChoices.length + (activeQuestion.allowCustomAnswer ? 1 : 0) : 0

  const terminalRows = stdout.rows || 40
  const terminalColumns = stdout.columns || 80
  const activeTaskRows = Math.min(3, tasks.filter(task => task.status !== 'done' && task.status !== 'canceled').length)
  // Heights count border (2) + title (1) + each panel's content lines exactly;
  // questions get slack for prompt wrapping.
  const inputPanelHeight = (() => {
    const cap = Math.max(10, terminalRows - 8)
    if (mode === 'chat') return 8
    if (mode === 'questions') return Math.min(8 + Math.max(1, questionRows), cap)
    if (mode === 'team') return Math.min(6 + Math.max(1, coworkerNames.length) + activeTaskRows, cap)
    return 5
  })()
  const logPanelHeight = Math.max(8, terminalRows - inputPanelHeight)
  const visibleLogCount = Math.max(1, logPanelHeight - 3)
  const logWrapWidth = logContentWidth(terminalColumns)
  const displayLines = useMemo(() => expandLogLines(logs, logWrapWidth), [logs, logWrapWidth])
  const pinnedName = names[Math.min(pinnedIndex, Math.max(0, names.length - 1))] ?? 'All'
  const target = useMemo(() => parseMessageTarget(message, names, pinnedName), [message, names, pinnedName])

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

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset the answer state whenever the active question changes
  useEffect(() => {
    setSelectedChoices([])
    setCustomAnswer('')
    setHighlightIndex(0)
  }, [questionIndex])

  useEffect(() => {
    const officePoll = setInterval(() => {
      setStatuses(listCoworkerStatuses())
      setTasks(listOfficeTasks())
      setQuestions(listHumanQuestions({ unansweredOnly: true }))
      const unread = listHumanInbox({ unreadOnly: true })
      if (unread.length === 0) return
      for (const inboxMessage of unread) addLog(`📬 ${inboxMessage.from} → you: ${inboxMessage.body}`, 'green')
      markAllHumanMessagesSeen()
    }, 1_000)
    return () => clearInterval(officePoll)
  }, [addLog])

  function refreshRoster() {
    setNames(['All', ...listAgentCoworkers().map(profile => profile.name)])
    setPinnedIndex(0)
    setTeamIndex(0)
  }

  function hire(name: string, role: string) {
    try {
      const existing = listAgentCoworkers().map(profile => profile.name)
      const hired = hireCoworker({ name, role })
      for (const coworker of existing) {
        officeMailbox.send({
          from: 'office-admin',
          to: coworker,
          body: `Coworker directory updated: ${hired.profile.name} joined as ${hired.profile.role}. Use list_coworkers for the current full list.`,
        })
      }
      setAgents(current => ({ ...current, [hired.profile.name]: hired.agent }))
      refreshRoster()
      addLog(`➕ Hired ${hired.profile.name}: ${hired.profile.role}`, 'green')
    } catch (error) {
      addLog(`❌ Could not hire coworker: ${error instanceof Error ? error.message : String(error)}`, 'red')
    }
  }

  function fire(name: string) {
    const removed = fireCoworker(name)
    if (!removed) return
    unwatchAgent(removed.name)
    setAgents(current => {
      const { [removed.name]: _removed, ...rest } = current
      return rest
    })
    refreshRoster()
    addLog(`🗑️ Fired ${removed.name} for this run`, 'red')
  }

  function applyTeam(team: TeamFile, sourcePath: string) {
    for (const existing of listAgentCoworkers()) {
      unwatchAgent(existing.name)
      fireCoworker(existing.name)
    }
    const nextAgents: Record<string, Agent> = {}
    const failures: string[] = []
    for (const member of team.coworkers) {
      try {
        const hired = hireCoworker(member)
        nextAgents[hired.profile.name] = hired.agent
      } catch (error) {
        failures.push(`${member.name} (${error instanceof Error ? error.message : String(error)})`)
      }
    }
    setAgents(nextAgents)
    refreshRoster()
    addLog(`📂 Loaded team${team.name ? ` "${team.name}"` : ''} from ${sourcePath} — ${listAgentCoworkers().length} coworker(s)`, 'green')
    for (const failure of failures) addLog(`⚠️ Skipped ${failure}`, 'yellow')
  }

  function saveTeam(path: string) {
    try {
      saveTeamFile(path, { coworkers: listAgentCoworkers() })
      setTeamFile(path)
      addLog(`💾 Saved team (${listAgentCoworkers().length} coworker(s)) to ${path}`, 'green')
    } catch (error) {
      addLog(`❌ Could not save team: ${error instanceof Error ? error.message : String(error)}`, 'red')
    }
  }

  function loadTeam(path: string) {
    try {
      const team = loadTeamFile(path)
      applyTeam(team, path)
      setTeamFile(path)
    } catch (error) {
      addLog(`❌ Could not load team: ${error instanceof Error ? error.message : String(error)}`, 'red')
    }
  }

  function sendChatMessage() {
    const { to, body } = target
    if (!body) return
    if (coworkerNames.length === 0) {
      addLog('🪑 The office is empty — press Tab and hire coworkers before sending messages.', 'yellow')
      return
    }
    setHistory(current => [...current.slice(-(MESSAGE_HISTORY_LIMIT - 1)), message])
    setHistoryIndex(null)
    setMessage('')
    deliveryQueue.current = deliveryQueue.current
      .then(() => deliverHumanMessage(to, body))
      .then(sent => addLog(`🧑 You → ${to}: ${sent.body}`, 'cyan'))
      .catch(error => addLog(`❌ Could not deliver to ${to}: ${error instanceof Error ? error.message : String(error)}`, 'red'))
  }

  function submitQuestionAnswer(question: HumanQuestion) {
    const choices = question.choices.slice(0, 9)
    const trimmedCustom = customAnswer.trim()
    let finalChoices = selectedChoices
    if (finalChoices.length === 0 && !trimmedCustom) {
      // Convenience: in single mode, Enter answers with the highlighted choice.
      if (question.mode === 'single' && highlightIndex < choices.length && choices[highlightIndex]) {
        finalChoices = [choices[highlightIndex]!]
      } else {
        return
      }
    }
    try {
      answerHumanQuestion({ id: question.id, selectedChoices: finalChoices, customAnswer: trimmedCustom || undefined })
      addLog(`✅ You answered ${question.from}'s question`, 'green')
      setSelectedChoices([])
      setCustomAnswer('')
      setHighlightIndex(0)
      setQuestions(listHumanQuestions({ unansweredOnly: true }))
      if (questions.length <= 1) setMode('chat')
    } catch (error) {
      addLog(`❌ Could not answer question: ${error instanceof Error ? error.message : String(error)}`, 'red')
    }
  }

  useInput((input, key) => {
    if (key.ctrl && input === 'c') exit()

    if ((key as any).pageUp) return setLogScrollOffset(offset => Math.min(Math.max(0, displayLines.length - visibleLogCount), offset + visibleLogCount))
    if ((key as any).pageDown) return setLogScrollOffset(offset => Math.max(0, offset - visibleLogCount))
    if ((key as any).end) return setLogScrollOffset(0)
    if ((key as any).home) return setLogScrollOffset(Math.max(0, displayLines.length - visibleLogCount))

    if (key.tab && (mode === 'chat' || mode === 'team' || mode === 'questions')) {
      setMode(current => (current === 'chat' ? 'team' : 'chat'))
      return
    }

    // "?" opens questions only where it cannot be part of typed text.
    if (input === '?' && questions.length > 0 && ((mode === 'chat' && message === '') || mode === 'team')) {
      setMode('questions')
      return
    }

    if (mode === 'questions') {
      if (key.escape) return setMode('chat')
      if (key.leftArrow) return setQuestionIndex(index => Math.max(0, index - 1))
      if (key.rightArrow) return setQuestionIndex(index => Math.min(Math.max(0, questions.length - 1), index + 1))
      if (!activeQuestion) return
      const rowCount = questionChoices.length + (activeQuestion.allowCustomAnswer ? 1 : 0)
      const onCustomRow = activeQuestion.allowCustomAnswer && highlightIndex === questionChoices.length
      if (key.upArrow) return setHighlightIndex(index => Math.max(0, index - 1))
      if (key.downArrow) return setHighlightIndex(index => Math.min(Math.max(0, rowCount - 1), index + 1))
      if (key.return) return submitQuestionAnswer(activeQuestion)
      if (onCustomRow) {
        if (key.backspace || key.delete) return setCustomAnswer(value => value.slice(0, -1))
        if (input && !key.ctrl && !key.meta) {
          if (activeQuestion.mode === 'single') setSelectedChoices([])
          setCustomAnswer(value => value + input)
        }
        return
      }
      if (input === ' ') {
        const choice = questionChoices[highlightIndex]
        if (!choice) return
        if (activeQuestion.mode === 'single') {
          setSelectedChoices([choice])
          setCustomAnswer('')
        } else {
          setSelectedChoices(current => current.includes(choice) ? current.filter(item => item !== choice) : [...current, choice])
        }
      }
      return
    }

    if (mode === 'add-name' || mode === 'add-role') {
      const setter = mode === 'add-name' ? setDraftName : setDraftRole
      if (key.escape) {
        setMode('team')
        setDraftName('')
        setDraftRole('')
        return
      }
      if (key.return) {
        if (mode === 'add-name') {
          if (draftName.trim()) setMode('add-role')
        } else {
          hire(draftName, draftRole)
          setDraftName('')
          setDraftRole('')
          setMode('team')
        }
        return
      }
      if (key.backspace || key.delete) return setter(value => value.slice(0, -1))
      if (input && !key.ctrl && !key.meta) setter(value => value + input)
      return
    }

    if (mode === 'save-team' || mode === 'load-team') {
      if (key.escape) return setMode('team')
      if (key.return) {
        const path = draftFile.trim()
        if (!path) return
        if (mode === 'save-team') saveTeam(path)
        else loadTeam(path)
        setMode('team')
        return
      }
      if (key.backspace || key.delete) return setDraftFile(value => value.slice(0, -1))
      if (input && !key.ctrl && !key.meta) setDraftFile(value => value + input)
      return
    }

    if (mode === 'team') {
      if (key.escape) return setMode('chat')
      if (key.upArrow) return setTeamIndex(index => coworkerNames.length === 0 ? 0 : (index - 1 + coworkerNames.length) % coworkerNames.length)
      if (key.downArrow) return setTeamIndex(index => coworkerNames.length === 0 ? 0 : (index + 1) % coworkerNames.length)
      if (input === 'a') {
        setDraftName('')
        setDraftRole('')
        setMode('add-name')
        return
      }
      if (input === 'd') {
        const name = coworkerNames[Math.min(teamIndex, Math.max(0, coworkerNames.length - 1))]
        if (name) fire(name)
        return
      }
      if (input === 's') {
        setDraftFile(teamFile)
        setMode('save-team')
        return
      }
      if (input === 'l') {
        setDraftFile(teamFile)
        setMode('load-team')
        return
      }
      return
    }

    // Chat mode.
    if (key.escape) {
      setMessage('')
      setHistoryIndex(null)
      return
    }
    if (key.leftArrow) return setPinnedIndex(index => (index - 1 + names.length) % names.length)
    if (key.rightArrow) return setPinnedIndex(index => (index + 1) % names.length)
    if (key.upArrow) {
      if (history.length === 0) return
      const index = historyIndex === null ? history.length - 1 : Math.max(0, historyIndex - 1)
      setHistoryIndex(index)
      setMessage(history[index]!)
      return
    }
    if (key.downArrow) {
      if (historyIndex === null) return
      const index = historyIndex + 1
      if (index >= history.length) {
        setHistoryIndex(null)
        setMessage('')
      } else {
        setHistoryIndex(index)
        setMessage(history[index]!)
      }
      return
    }
    if (key.return) return sendChatMessage()
    if (key.backspace || key.delete) return setMessage(value => value.slice(0, -1))
    if (input && !key.ctrl && !key.meta) {
      setHistoryIndex(null)
      setMessage(value => value + input)
    }
  })

  const maxScrollOffset = Math.max(0, displayLines.length - visibleLogCount)
  const effectiveScrollOffset = Math.min(logScrollOffset, maxScrollOffset)
  const logEnd = displayLines.length - effectiveScrollOffset
  const logStart = Math.max(0, logEnd - visibleLogCount)
  const visibleLogs = useMemo(() => displayLines.slice(logStart, logEnd), [displayLines, logStart, logEnd])

  const panelTitle =
    mode === 'chat' ? 'Office chat'
    : mode === 'team' ? 'Team'
    : mode === 'questions' ? `Questions from coworkers`
    : mode === 'save-team' ? 'Save team'
    : mode === 'load-team' ? 'Load team'
    : 'Hire coworker'
  const borderColor = mode === 'chat' ? 'cyan' : mode === 'questions' ? 'yellow' : 'green'

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

      <Box borderStyle="round" borderColor={borderColor} paddingX={1} flexDirection="column" height={inputPanelHeight}>
        <Text bold>{panelTitle}</Text>
        {mode === 'chat' && <ChatPanel names={names} pinnedIndex={pinnedIndex} target={target} message={message} statuses={statuses} tasks={tasks} questionCount={questions.length} />}
        {mode === 'team' && <TeamPanel coworkerNames={coworkerNames} teamIndex={teamIndex} statuses={statuses} tasks={tasks} teamFile={teamFile} />}
        {mode === 'questions' && <QuestionPanel questions={questions} questionIndex={questionIndex} highlightIndex={highlightIndex} selectedChoices={selectedChoices} customAnswer={customAnswer} />}
        {mode === 'add-name' && <><Text>Coworker name: {draftName}<Text color="gray">█</Text></Text><Text color="gray">Enter continue · Esc cancel</Text></>}
        {mode === 'add-role' && <><Text>Role for {draftName}: {draftRole}<Text color="gray">█</Text></Text><Text color="gray">Enter hire · Esc cancel</Text></>}
        {mode === 'save-team' && <><Text>Save team to: {draftFile}<Text color="gray">█</Text></Text><Text color="gray">Enter save · Esc cancel</Text></>}
        {mode === 'load-team' && <><Text>Load team from: {draftFile}<Text color="gray">█</Text></Text><Text color="gray">Enter load (replaces current team) · Esc cancel</Text></>}
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

function bootstrapTeam(): { teamFile: string; logs: { text: string; color?: string }[] } {
  const arg = process.argv[2]
  const path = arg ?? (existsSync(DEFAULT_TEAM_FILE) ? DEFAULT_TEAM_FILE : undefined)
  if (!path) return { teamFile: DEFAULT_TEAM_FILE, logs: [] }

  try {
    const team = loadTeamFile(path)
    const failures: string[] = []
    for (const member of team.coworkers) {
      try {
        hireCoworker(member)
      } catch (error) {
        failures.push(`${member.name} (${error instanceof Error ? error.message : String(error)})`)
      }
    }
    return {
      teamFile: path,
      logs: [
        { text: `📂 Loaded team${team.name ? ` "${team.name}"` : ''} from ${path} — ${listAgentCoworkers().length} coworker(s)`, color: 'green' },
        ...failures.map(failure => ({ text: `⚠️ Skipped ${failure}`, color: 'yellow' })),
      ],
    }
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error)
    if (arg) {
      // The user explicitly asked for this file; failing loudly beats a silent empty office.
      console.error(reason)
      process.exit(1)
    }
    return { teamFile: DEFAULT_TEAM_FILE, logs: [{ text: `⚠️ ${reason}`, color: 'yellow' }] }
  }
}

ensureModelProviderConfigured()
const bootstrap = bootstrapTeam()
render(<App initialTeamFile={bootstrap.teamFile} initialLogs={bootstrap.logs} />)
