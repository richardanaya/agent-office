import { createId } from './ids.js'
import { officeMailbox } from './mailbox.js'

export type HumanQuestionChoiceMode = 'single' | 'multiple'

export type HumanQuestion = {
  id: string
  from: string
  prompt: string
  mode: HumanQuestionChoiceMode
  choices: string[]
  allowCustomAnswer: boolean
  createdAt: string
  answeredAt?: string
  answer?: {
    selectedChoices: string[]
    customAnswer?: string
  }
}

const MAX_ANSWERED_QUESTIONS = 200

const questions: HumanQuestion[] = []

// Cap history: drop the oldest answered questions; unanswered ones are kept.
function pruneAnsweredQuestions() {
  let answeredCount = questions.filter(question => question.answeredAt).length
  for (let index = 0; index < questions.length && answeredCount > MAX_ANSWERED_QUESTIONS; ) {
    if (questions[index]!.answeredAt) {
      questions.splice(index, 1)
      answeredCount--
    } else {
      index++
    }
  }
}

export function askHumanQuestion(input: {
  from: string
  prompt: string
  choices: string[]
  mode: HumanQuestionChoiceMode
  allowCustomAnswer?: boolean
}): HumanQuestion {
  const question: HumanQuestion = {
    id: createId('question'),
    from: input.from,
    prompt: input.prompt.trim(),
    mode: input.mode,
    choices: input.choices.map(choice => choice.trim()).filter(Boolean),
    allowCustomAnswer: input.allowCustomAnswer ?? true,
    createdAt: new Date().toISOString(),
  }
  questions.push(question)
  pruneAnsweredQuestions()
  return question
}

export function listHumanQuestions(options: { unansweredOnly?: boolean } = {}): HumanQuestion[] {
  return questions.filter(question => !options.unansweredOnly || !question.answeredAt)
}

export function answerHumanQuestion(input: { id: string; selectedChoices?: string[]; customAnswer?: string }): HumanQuestion {
  const question = questions.find(item => item.id === input.id)
  if (!question) throw new Error(`No human question found with id ${input.id}.`)
  if (question.answeredAt) throw new Error(`Question ${input.id} has already been answered.`)

  const selectedChoices = (input.selectedChoices ?? []).filter(choice => question.choices.includes(choice))
  const customAnswer = input.customAnswer?.trim() || undefined
  question.answeredAt = new Date().toISOString()
  question.answer = { selectedChoices, customAnswer }

  const parts = []
  if (selectedChoices.length > 0) parts.push(`selected: ${selectedChoices.join(', ')}`)
  if (customAnswer) parts.push(`custom answer: ${customAnswer}`)
  // Leave this Human message undelivered so OfficeSignals wakes the requesting agent.
  officeMailbox.send({
    from: 'Human',
    to: question.from,
    body: `Answer to question "${question.prompt}": ${parts.length > 0 ? parts.join('; ') : 'no answer provided'}`,
  })

  return question
}
