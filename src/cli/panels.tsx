import { Box, Text } from 'ink'
import type { CoworkerStatus, OfficeTask } from '../office/kanban.js'
import type { HumanQuestion } from '../office/human-questions.js'
import type { MessageTarget } from './message-target.js'

export function statusFor(statuses: CoworkerStatus[], name: string) {
  return statuses.find(status => status.name.toLowerCase() === name.toLowerCase())
}

export function kanbanSummary(tasks: OfficeTask[]) {
  const columns = ['todo', 'doing', 'blocked', 'review', 'done']
  return columns.map(column => `${column}:${tasks.filter(task => task.status === column).length}`).join(' ')
}

export function ChatPanel({ names, pinnedIndex, target, message, statuses, tasks, questionCount }: {
  names: string[]
  pinnedIndex: number
  target: MessageTarget
  message: string
  statuses: CoworkerStatus[]
  tasks: OfficeTask[]
  questionCount: number
}) {
  const hasCoworkers = names.length > 1
  return <>
    <Text color="gray">Tab team{questionCount > 0 ? ` · ? answer ${questionCount} question${questionCount === 1 ? '' : 's'}` : ''} · ←/→ target · @name for a DM · ↑/↓ history · Ctrl+C quit</Text>
    {hasCoworkers ? (
      <Box marginTop={1}>{names.map((name, index) => {
        const status = statusFor(statuses, name)
        const label = `${name}${status ? `:${status.status}` : ''}`
        const pinned = index === pinnedIndex && !target.mentioned
        return <Box key={name} marginRight={2}><Text color={pinned ? 'cyan' : undefined} inverse={pinned}>{pinned ? ` ${label} ` : label}</Text></Box>
      })}</Box>
    ) : (
      <Box marginTop={1}><Text color="yellow">The office is empty. Press Tab to build your team — hire coworkers or load one from a file.</Text></Box>
    )}
    <Text color="gray">Kanban {kanbanSummary(tasks)}</Text>
    <Box>
      <Text color={target.mentioned ? 'magenta' : 'cyan'}>To {target.to}{target.mentioned ? ' (@)' : ''} ▸ </Text>
      <Text>{message}</Text>
      <Text color="gray">█</Text>
    </Box>
  </>
}

export function QuestionPanel({ questions, questionIndex, highlightIndex, selectedChoices, customAnswer }: {
  questions: HumanQuestion[]
  questionIndex: number
  highlightIndex: number
  selectedChoices: string[]
  customAnswer: string
}) {
  const question = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
  if (!question) return <Text color="gray">No pending questions. Esc returns to chat.</Text>
  const customActive = customAnswer.trim() !== ''
  const choices = question.choices.slice(0, 9)
  const canSubmit = customActive || selectedChoices.length > 0 || (question.mode === 'single' && highlightIndex < choices.length)
  return <>
    <Text color="gray">↑/↓ move · Space select · type on Custom to answer freely · Enter submit · ←/→ question · Esc chat</Text>
    <Text color="yellow" bold>{question.from} asks ({questionIndex + 1}/{questions.length}): {question.prompt}</Text>
    <Text color="gray">{question.mode === 'multiple' ? 'Pick any that apply' : 'Pick one'}{canSubmit ? ' · Enter to submit' : ''}</Text>
    {choices.map((choice, index) => {
      const highlighted = index === highlightIndex
      const selected = selectedChoices.includes(choice)
      return (
        <Text key={choice} color={selected ? 'green' : highlighted ? 'cyan' : undefined}>
          {highlighted ? '❯ ' : '  '}{selected ? '◉' : '○'} {choice}
        </Text>
      )
    })}
    {question.allowCustomAnswer && (
      <Text color={customActive ? 'green' : highlightIndex === choices.length ? 'cyan' : 'gray'}>
        {highlightIndex === choices.length ? '❯ ' : '  '}{customActive ? '◉' : '○'} Custom: {customAnswer}{highlightIndex === choices.length ? <Text color="gray">█</Text> : ''}
      </Text>
    )}
  </>
}

export function TeamPanel({ coworkerNames, teamIndex, statuses, tasks, teamFile }: {
  coworkerNames: string[]
  teamIndex: number
  statuses: CoworkerStatus[]
  tasks: OfficeTask[]
  teamFile: string
}) {
  const activeTasks = tasks.filter(task => task.status !== 'done' && task.status !== 'canceled').slice(0, 3)
  return <>
    <Text color="gray">a hire · d fire · s save team · l load team · ↑/↓ select · Esc/Tab chat</Text>
    {coworkerNames.length === 0 ? (
      <Text color="yellow">No coworkers yet. Press a to hire your first coworker, or l to load a team file.</Text>
    ) : (
      coworkerNames.map((name, index) => {
        const status = statusFor(statuses, name)
        return <Text key={name} color={index === teamIndex ? 'green' : undefined} inverse={index === teamIndex}>{index === teamIndex ? ` ${name} ` : ` ${name}`}<Text color="gray"> {status ? `${status.status}${status.note ? ` — ${status.note}` : ''}` : 'available'}</Text></Text>
      })
    )}
    <Text color="gray">Team file: {teamFile} · Kanban {kanbanSummary(tasks)}</Text>
    {activeTasks.map(task => <Text key={task.id}>• [{task.status}] {task.title}{task.assignee ? ` @${task.assignee}` : ''}</Text>)}
  </>
}
