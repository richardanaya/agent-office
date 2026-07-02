#!/usr/bin/env node
import { existsSync } from 'node:fs'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Box, Text, render, useApp, useInput, useStdout } from 'ink'
import { describeModelConfiguration } from './agents/model.js'
import { resolveOfficePort } from './config.js'
import { loadTeamFile } from './office/team.js'
import type { CoworkerStatus, OfficeTask } from './office/kanban.js'
import type { HumanQuestion } from './office/human-questions.js'
import type { OfficeState } from './protocol.js'
import { OfficeClient } from './client/office-client.js'
import { startOfficeServer, type OfficeServer } from './server/office-server.js'
import { expandLogLines, logContentWidth, type LogLine } from './cli/log-text.js'
import { formatOfficeEvent, handleReasoningChunk, type Chunk } from './cli/office-events.js'
import { parseMessageTarget } from './cli/message-target.js'
import { ChatPanel, QuestionPanel, TeamPanel } from './cli/panels.js'

type Mode = 'chat' | 'team' | 'questions' | 'add-name' | 'add-role' | 'save-team' | 'load-team'

const DEFAULT_TEAM_FILE = 'team.json'
const MESSAGE_HISTORY_LIMIT = 50
const STATE_POLL_INTERVAL_MS = 1_000

let nextLogId = 1

type InitialLog = { text: string; color?: string }

function App({ client, initialState, initialLogs }: { client: OfficeClient; initialState: OfficeState; initialLogs: InitialLog[] }) {
  const { exit } = useApp()
  const { stdout } = useStdout()
  const [mode, setMode] = useState<Mode>(() => (initialState.coworkers.length === 0 ? 'team' : 'chat'))
  const [names, setNames] = useState<string[]>(() => ['All', ...initialState.coworkers.map(profile => profile.name)])
  const [pinnedIndex, setPinnedIndex] = useState(0)
  const [teamIndex, setTeamIndex] = useState(0)
  const [message, setMessage] = useState('')
  const [history, setHistory] = useState<string[]>([])
  const [historyIndex, setHistoryIndex] = useState<number | null>(null)
  const [draftName, setDraftName] = useState('')
  const [draftRole, setDraftRole] = useState('')
  const [teamFile, setTeamFile] = useState(initialState.teamFile ?? DEFAULT_TEAM_FILE)
  const [draftFile, setDraftFile] = useState('')
  const [logScrollOffset, setLogScrollOffset] = useState(0)
  const [statuses, setStatuses] = useState<CoworkerStatus[]>(initialState.statuses)
  const [tasks, setTasks] = useState<OfficeTask[]>(initialState.tasks)
  const [questions, setQuestions] = useState<HumanQuestion[]>(initialState.questions)
  const [questionIndex, setQuestionIndex] = useState(0)
  const [highlightIndex, setHighlightIndex] = useState(0)
  const [selectedChoices, setSelectedChoices] = useState<string[]>([])
  const [customAnswer, setCustomAnswer] = useState('')
  const connected = useRef(true)
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

  const refreshState = useCallback(async () => {
    try {
      const state = await client.getState()
      setStatuses(state.statuses)
      setTasks(state.tasks)
      setQuestions(state.questions)
      setNames(['All', ...state.coworkers.map(profile => profile.name)])
      if (state.teamFile) setTeamFile(state.teamFile)
    } catch {
      // Connection loss is reported by the event stream status callback.
    }
  }, [client])

  useEffect(() => {
    const poll = setInterval(() => void refreshState(), STATE_POLL_INTERVAL_MS)
    return () => clearInterval(poll)
  }, [refreshState])

  useEffect(() => {
    const stop = client.subscribeEvents(({ event }) => {
      switch (event.type) {
        case 'agent-event': {
          const chunk = event.chunk as Chunk
          if (handleReasoningChunk(event.agent, chunk, addLog)) return
          const line = formatOfficeEvent(event.agent, chunk)
          if (line) addLog(line.text, line.color)
          return
        }
        case 'human-message':
          addLog(`📬 ${event.message.from} → you: ${event.message.body}`, 'green')
          return
        case 'human-message-sent':
          addLog(`🧑 You → ${event.to}: ${event.body}`, 'cyan')
          return
        case 'roster-changed':
          void refreshState()
          return
      }
    }, {
      onStatus: status => {
        if (status === 'disconnected' && connected.current) {
          connected.current = false
          addLog('⚠️ Lost connection to the office server — retrying…', 'red')
        } else if (status === 'connected' && !connected.current) {
          connected.current = true
          addLog('✅ Reconnected to the office server.', 'green')
          void refreshState()
        }
      },
    })
    return stop
  }, [client, addLog, refreshState])

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset the answer state whenever the active question changes
  useEffect(() => {
    setSelectedChoices([])
    setCustomAnswer('')
    setHighlightIndex(0)
  }, [questionIndex])

  function hire(name: string, role: string) {
    client.hire(name, role)
      .then(({ profile }) => {
        addLog(`➕ Hired ${profile.name}: ${profile.role}`, 'green')
        void refreshState()
      })
      .catch(error => addLog(`❌ Could not hire coworker: ${error instanceof Error ? error.message : String(error)}`, 'red'))
  }

  function fire(name: string) {
    client.fire(name)
      .then(({ profile }) => {
        addLog(`🗑️ Fired ${profile.name} for this run`, 'red')
        setTeamIndex(0)
        setPinnedIndex(0)
        void refreshState()
      })
      .catch(error => addLog(`❌ Could not fire ${name}: ${error instanceof Error ? error.message : String(error)}`, 'red'))
  }

  function saveTeam(path: string) {
    client.saveTeam(path)
      .then(result => {
        setTeamFile(result.path)
        addLog(`💾 Saved team (${result.coworkers.length} coworker(s)) to ${result.path} on the server`, 'green')
      })
      .catch(error => addLog(`❌ Could not save team: ${error instanceof Error ? error.message : String(error)}`, 'red'))
  }

  function loadTeam(path: string) {
    client.loadTeam(path)
      .then(result => {
        setTeamFile(result.path)
        addLog(`📂 Loaded team${result.name ? ` "${result.name}"` : ''} from ${result.path} — ${result.coworkers.length} coworker(s)`, 'green')
        for (const skipped of result.skipped) addLog(`⚠️ Skipped ${skipped}`, 'yellow')
        setTeamIndex(0)
        setPinnedIndex(0)
        void refreshState()
      })
      .catch(error => addLog(`❌ Could not load team: ${error instanceof Error ? error.message : String(error)}`, 'red'))
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
    // The server broadcasts a human-message-sent event to all clients, so
    // success shows up through the event stream rather than a local log.
    deliveryQueue.current = deliveryQueue.current
      .then(() => client.sendMessage(to, body))
      .then(() => undefined)
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
    client.answerQuestion(question.id, { selectedChoices: finalChoices, customAnswer: trimmedCustom || undefined })
      .then(() => {
        addLog(`✅ You answered ${question.from}'s question`, 'green')
        setSelectedChoices([])
        setCustomAnswer('')
        setHighlightIndex(0)
        if (questions.length <= 1) setMode('chat')
        void refreshState()
      })
      .catch(error => addLog(`❌ Could not answer question: ${error instanceof Error ? error.message : String(error)}`, 'red'))
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
    : mode === 'questions' ? 'Questions from coworkers'
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
        {mode === 'team' && <TeamPanel coworkerNames={coworkerNames} teamIndex={Math.min(teamIndex, Math.max(0, coworkerNames.length - 1))} statuses={statuses} tasks={tasks} teamFile={teamFile} />}
        {mode === 'questions' && <QuestionPanel questions={questions} questionIndex={questionIndex} highlightIndex={highlightIndex} selectedChoices={selectedChoices} customAnswer={customAnswer} />}
        {mode === 'add-name' && <><Text>Coworker name: {draftName}<Text color="gray">█</Text></Text><Text color="gray">Enter continue · Esc cancel</Text></>}
        {mode === 'add-role' && <><Text>Role for {draftName}: {draftRole}<Text color="gray">█</Text></Text><Text color="gray">Enter hire · Esc cancel</Text></>}
        {mode === 'save-team' && <><Text>Save team to (server path): {draftFile}<Text color="gray">█</Text></Text><Text color="gray">Enter save · Esc cancel</Text></>}
        {mode === 'load-team' && <><Text>Load team from (server path): {draftFile}<Text color="gray">█</Text></Text><Text color="gray">Enter load (replaces current team) · Esc cancel</Text></>}
      </Box>
    </Box>
  )
}

type LaunchPlan =
  | { kind: 'standalone'; teamFile?: string }
  | { kind: 'serve'; teamFile?: string; port: number; host: string }
  | { kind: 'connect'; url: string }

function printUsage(code: number): never {
  const out = code === 0 ? console.log : console.error
  out(`Usage:
  agent-office [team.json]              run the office with a terminal UI
  agent-office serve [team.json]        run a headless office server
    --port <port>                       port to listen on (default ${resolveOfficePortSafe()})
    --host <host>                       host to bind (default 127.0.0.1)
  agent-office connect <url>            attach a terminal UI to a running server

If ./team.json exists it loads automatically. AGENT_OFFICE_PORT overrides the default port.`)
  process.exit(code)
}

function resolveOfficePortSafe() {
  try {
    return resolveOfficePort()
  } catch {
    return 4747
  }
}

function parseCliArgs(argv: string[]): LaunchPlan {
  const [first, ...rest] = argv
  if (first === '--help' || first === '-h') printUsage(0)
  if (first === 'connect') {
    if (!rest[0]) printUsage(1)
    return { kind: 'connect', url: rest[0] }
  }
  if (first && /^https?:\/\//.test(first)) return { kind: 'connect', url: first }
  if (first === 'serve') {
    let port = resolveOfficePort()
    let host = '127.0.0.1'
    let teamFile: string | undefined
    for (let index = 0; index < rest.length; index++) {
      const arg = rest[index]!
      if (arg === '--port') {
        port = Number.parseInt(rest[++index] ?? '', 10)
        if (!Number.isInteger(port) || port < 0 || port > 65535) printUsage(1)
      } else if (arg === '--host') {
        const value = rest[++index]
        if (!value) printUsage(1)
        host = value
      } else if (!arg.startsWith('-') && teamFile === undefined) {
        teamFile = arg
      } else {
        printUsage(1)
      }
    }
    return { kind: 'serve', port, host, teamFile }
  }
  if (first?.startsWith('-')) printUsage(1)
  return { kind: 'standalone', teamFile: first }
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

// Validate the team file up front so server startup errors are port errors
// only; an explicitly requested file fails loudly, an auto-detected one warns.
function resolveTeamFile(explicit: string | undefined): { teamFile?: string; warnings: InitialLog[] } {
  const candidate = explicit ?? (existsSync(DEFAULT_TEAM_FILE) ? DEFAULT_TEAM_FILE : undefined)
  if (!candidate) return { warnings: [] }
  try {
    loadTeamFile(candidate)
    return { teamFile: candidate, warnings: [] }
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error)
    if (explicit) {
      console.error(reason)
      process.exit(1)
    }
    return { warnings: [{ text: `⚠️ ${reason}`, color: 'yellow' }] }
  }
}

async function main() {
  const plan = parseCliArgs(process.argv.slice(2))

  if (plan.kind === 'connect') {
    const client = new OfficeClient(plan.url)
    let initialState: OfficeState
    try {
      initialState = await client.getState()
    } catch (error) {
      console.error(`Could not reach an office server at ${plan.url}: ${error instanceof Error ? error.message : String(error)}`)
      process.exit(1)
    }
    render(<App client={client} initialState={initialState} initialLogs={[{ text: `🔌 Connected to office server at ${client.baseUrl}`, color: 'gray' }]} />)
    return
  }

  ensureModelProviderConfigured()
  const { teamFile, warnings } = resolveTeamFile(plan.teamFile)

  if (plan.kind === 'serve') {
    let server: OfficeServer
    try {
      server = await startOfficeServer({ port: plan.port, host: plan.host, teamFile })
    } catch (error) {
      console.error(`Could not start office server: ${error instanceof Error ? error.message : String(error)}`)
      process.exit(1)
    }
    for (const warning of warnings) console.warn(warning.text)
    for (const warning of server.bootstrapWarnings) console.warn(`⚠️ ${warning}`)
    console.log(`agent-office server listening on ${server.url}${teamFile ? ` (team: ${teamFile})` : ''}`)
    console.log(`Attach a terminal with: agent-office connect ${server.url}`)
    return
  }

  let server: OfficeServer
  const preferredPort = resolveOfficePort()
  try {
    server = await startOfficeServer({ port: preferredPort, host: '127.0.0.1', teamFile })
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'EADDRINUSE') {
      server = await startOfficeServer({ port: 0, host: '127.0.0.1', teamFile })
      warnings.push({ text: `⚠️ Port ${preferredPort} is busy — office server is on port ${server.port} instead.`, color: 'yellow' })
    } else {
      console.error(`Could not start office server: ${error instanceof Error ? error.message : String(error)}`)
      process.exit(1)
    }
  }

  const client = new OfficeClient(server.url)
  const initialState = await client.getState()
  const initialLogs: InitialLog[] = [
    { text: `🌐 Office server at ${server.url} — attach another client with: agent-office connect ${server.url}`, color: 'gray' },
    ...warnings,
    ...server.bootstrapWarnings.map(text => ({ text: `⚠️ ${text}`, color: 'yellow' })),
    ...(teamFile ? [{ text: `📂 Loaded team from ${teamFile} — ${initialState.coworkers.length} coworker(s)`, color: 'green' }] : []),
  ]
  render(<App client={client} initialState={initialState} initialLogs={initialLogs} />)
}

void main()
