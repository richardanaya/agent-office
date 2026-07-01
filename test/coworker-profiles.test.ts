import { describe, expect, it } from 'vitest'
import { addAgentCoworker, listAgentCoworkers, listCoworkerProfiles, removeAgentCoworker } from '../src/office/coworkers.js'

describe('coworker profiles', () => {
  it('adds a trimmed coworker and removes it again', () => {
    const added = addAgentCoworker({ name: ' Dana ', role: ' data analyst ' })
    expect(added.name).toBe('Dana')
    expect(added.role).toBe('data analyst')
    expect(listAgentCoworkers().some(profile => profile.name === 'Dana')).toBe(true)
    const removed = removeAgentCoworker(' dana ')
    expect(removed?.name).toBe('Dana')
    expect(listAgentCoworkers().some(profile => profile.name === 'Dana')).toBe(false)
  })

  it('rejects invalid names and roles', () => {
    expect(() => addAgentCoworker({ name: '1bad', role: 'valid role' })).toThrow(/must start with a letter/)
    expect(() => addAgentCoworker({ name: 'ok name with spaces', role: 'valid role' })).toThrow(/must start with a letter/)
    expect(() => addAgentCoworker({ name: 'Dana', role: 'x' })).toThrow(/3-200 characters/)
  })

  it('rejects duplicate names case-insensitively', () => {
    addAgentCoworker({ name: 'Ed', role: 'test engineer' })
    try {
      expect(() => addAgentCoworker({ name: 'ED', role: 'another role' })).toThrow(/already exists/)
    } finally {
      removeAgentCoworker('Ed')
    }
  })

  it('returns undefined when removing an unknown coworker', () => {
    expect(removeAgentCoworker('nobody')).toBeUndefined()
  })

  it('includes Human in the full profile list but not the agent list', () => {
    const all = listCoworkerProfiles().map(profile => profile.name)
    expect(all).toContain('Human')
    expect(listAgentCoworkers().map(profile => profile.name)).not.toContain('Human')
  })
})
