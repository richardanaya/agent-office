import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { clearScheduledAction, getScheduledAction, scheduleSelfWake } from '../src/office/scheduled-actions.js'
import { officeMailbox } from '../src/office/mailbox.js'

describe('scheduled actions', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    clearScheduledAction('Alice')
    vi.useRealTimers()
  })

  it('schedules a wake and exposes it via getScheduledAction', () => {
    const action = scheduleSelfWake({ agentName: 'Alice', instruction: 'Check replies', delaySeconds: 10 })
    expect(action.delaySeconds).toBe(10)
    expect(getScheduledAction(' alice ')?.id).toBe(action.id)
  })

  it('rejects a second wake while one is pending', () => {
    scheduleSelfWake({ agentName: 'Alice', instruction: 'Check replies', delaySeconds: 10 })
    expect(() => scheduleSelfWake({ agentName: 'alice', instruction: 'Another wake', delaySeconds: 5 })).toThrow(/already waiting/)
  })

  it('validates delay bounds and instruction length', () => {
    expect(() => scheduleSelfWake({ agentName: 'Alice', instruction: 'Valid instruction', delaySeconds: 0 })).toThrow(/1 to 3600/)
    expect(() => scheduleSelfWake({ agentName: 'Alice', instruction: 'Valid instruction', delaySeconds: 3601 })).toThrow(/1 to 3600/)
    expect(() => scheduleSelfWake({ agentName: 'Alice', instruction: 'Valid instruction', delaySeconds: 2.5 })).toThrow(/integer/)
    expect(() => scheduleSelfWake({ agentName: 'Alice', instruction: 'hey', delaySeconds: 10 })).toThrow(/between 5 and 1000/)
  })

  it('delivers the wake instruction through the mailbox and clears itself', () => {
    scheduleSelfWake({ agentName: 'Alice', instruction: 'Summarize replies', delaySeconds: 10 })
    const before = officeMailbox.undeliveredFor('alice').length
    vi.advanceTimersByTime(10_000)
    const messages = officeMailbox.undeliveredFor('alice')
    expect(messages.length).toBe(before + 1)
    expect(messages.at(-1)!.body).toContain('Summarize replies')
    expect(getScheduledAction('Alice')).toBeUndefined()
  })

  it('clearScheduledAction cancels the pending timer', () => {
    scheduleSelfWake({ agentName: 'Alice', instruction: 'Never fires', delaySeconds: 10 })
    const before = officeMailbox.undeliveredFor('alice').length
    const cleared = clearScheduledAction('alice')
    expect(cleared?.instruction).toBe('Never fires')
    vi.advanceTimersByTime(60_000)
    expect(officeMailbox.undeliveredFor('alice').length).toBe(before)
    expect(clearScheduledAction('alice')).toBeUndefined()
  })
})
