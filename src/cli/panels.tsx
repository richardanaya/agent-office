import { Box, Text } from 'ink'
import type { CoworkerStatus, OfficeTask } from '../office/kanban.js'
import type { HumanQuestion } from '../office/human-questions.js'

export function statusFor(statuses: CoworkerStatus[], name: string) {
  return statuses.find(status => status.name.toLowerCase() === name.toLowerCase())
}

export function kanbanSummary(tasks: OfficeTask[]) {
  const columns = ['todo', 'doing', 'blocked', 'review', 'done']
  return columns.map(column => `${column}:${tasks.filter(task => task.status === column).length}`).join(' ')
}

export function ChatPanel({ names, selectedIndex, selectedName, message, statuses, tasks, questionCount }: { names: string[]; selectedIndex: number; selectedName?: string; message: string; statuses: CoworkerStatus[]; tasks: OfficeTask[]; questionCount: number }) {
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

export function QuestionPanel({ questions, questionIndex, selectedChoices, customSelected, customAnswer }: { questions: HumanQuestion[]; questionIndex: number; selectedChoices: string[]; customSelected: boolean; customAnswer: string }) {
  const question = questions[Math.min(questionIndex, Math.max(0, questions.length - 1))]
  if (!question) return <Text color="gray">No pending questions. Esc/Tab returns to chat.</Text>
  return <>
    <Text>Esc {customSelected ? 'clear custom' : 'chat'} · ↑/↓ question · number toggles/selects · select Custom to type · Enter submit</Text>
    <Text color="yellow">{questionIndex + 1}/{questions.length} From {question.from}: {question.prompt}</Text>
    <Text color="gray">Mode: {question.mode} · custom answer allowed</Text>
    {question.choices.slice(0, 9).map((choice, index) => (
      <Text key={choice} color={selectedChoices.includes(choice) ? 'green' : undefined}>{index + 1}. {selectedChoices.includes(choice) ? '☑' : '☐'} {choice}</Text>
    ))}
    {question.allowCustomAnswer && <Text color={customSelected ? 'green' : undefined}>{question.choices.length + 1}. {customSelected ? '☑' : '☐'} Custom{customSelected ? `: ${customAnswer}` : ''}{customSelected ? <Text color="gray">█</Text> : ''}</Text>}
  </>
}

export function CoworkerPanel({ names, selectedIndex, statuses, tasks }: { names: string[]; selectedIndex: number; statuses: CoworkerStatus[]; tasks: OfficeTask[] }) {
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
