import { describe, expect, it } from 'vitest'
import { answerHumanQuestion, askHumanQuestion, listHumanQuestions } from '../src/office/human-questions.js'
import { officeMailbox } from '../src/office/mailbox.js'

function ask(overrides: Partial<Parameters<typeof askHumanQuestion>[0]> = {}) {
  return askHumanQuestion({
    from: 'Alice',
    prompt: 'Pick a color',
    choices: [' red ', 'blue', ''],
    mode: 'single',
    ...overrides,
  })
}

describe('askHumanQuestion', () => {
  it('trims the prompt and drops empty choices', () => {
    const question = ask({ prompt: '  Pick a color  ' })
    expect(question.prompt).toBe('Pick a color')
    expect(question.choices).toEqual(['red', 'blue'])
    expect(question.allowCustomAnswer).toBe(true)
    expect(question.answeredAt).toBeUndefined()
  })

  it('lists unanswered questions only when asked', () => {
    const question = ask()
    expect(listHumanQuestions({ unansweredOnly: true }).some(item => item.id === question.id)).toBe(true)
    answerHumanQuestion({ id: question.id, selectedChoices: ['red'] })
    expect(listHumanQuestions({ unansweredOnly: true }).some(item => item.id === question.id)).toBe(false)
    expect(listHumanQuestions().some(item => item.id === question.id)).toBe(true)
  })
})

describe('answerHumanQuestion', () => {
  it('records the answer and queues a mailbox reply to the asker', () => {
    const question = ask({ from: 'Bob' })
    const before = officeMailbox.undeliveredFor('bob').length
    const answered = answerHumanQuestion({ id: question.id, selectedChoices: ['blue'], customAnswer: ' extra ' })
    expect(answered.answer?.selectedChoices).toEqual(['blue'])
    expect(answered.answer?.customAnswer).toBe('extra')
    const replies = officeMailbox.undeliveredFor('bob')
    expect(replies.length).toBe(before + 1)
    expect(replies.at(-1)!.body).toContain('selected: blue')
    expect(replies.at(-1)!.body).toContain('custom answer: extra')
  })

  it('ignores choices that were not offered', () => {
    const question = ask()
    const answered = answerHumanQuestion({ id: question.id, selectedChoices: ['green', 'red'] })
    expect(answered.answer?.selectedChoices).toEqual(['red'])
  })

  it('throws for an unknown id', () => {
    expect(() => answerHumanQuestion({ id: 'missing' })).toThrow(/No human question/)
  })

  it('rejects answering the same question twice', () => {
    const question = ask()
    answerHumanQuestion({ id: question.id, selectedChoices: ['red'] })
    expect(() => answerHumanQuestion({ id: question.id, selectedChoices: ['blue'] })).toThrow(/already been answered/)
  })
})
