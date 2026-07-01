import { describe, expect, it } from 'vitest'
import { fireCoworker, hireCoworker } from '../src/office/hiring.js'
import { getCoworkerAgent } from '../src/agents/coworkers.js'
import { listAgentCoworkers } from '../src/office/coworkers.js'
import { mastra } from '../src/mastra/index.js'

describe('hiring', () => {
  it('hire registers the profile, agent, and Mastra entry; fire removes them all', () => {
    const hired = hireCoworker({ name: 'Frank', role: 'facilities manager' })
    try {
      expect(hired.profile.name).toBe('Frank')
      expect(listAgentCoworkers().some(profile => profile.name === 'Frank')).toBe(true)
      expect(getCoworkerAgent('frank')).toBe(hired.agent)
      expect(mastra.getAgentById('frank')).toBeTruthy()
    } finally {
      const removed = fireCoworker('Frank')
      expect(removed?.name).toBe('Frank')
    }
    expect(listAgentCoworkers().some(profile => profile.name === 'Frank')).toBe(false)
    expect(getCoworkerAgent('frank')).toBeUndefined()
    expect(() => mastra.getAgentById('frank')).toThrow()
  })

  it('rejects hiring a duplicate coworker', () => {
    hireCoworker({ name: 'Grace', role: 'graphics artist' })
    try {
      expect(() => hireCoworker({ name: 'grace', role: 'another role' })).toThrow(/already exists/)
    } finally {
      fireCoworker('Grace')
    }
  })

  it('fire returns undefined for unknown coworkers', () => {
    expect(fireCoworker('nobody')).toBeUndefined()
  })

  it('the office starts with no default coworkers', () => {
    expect(listAgentCoworkers()).toEqual([])
  })
})
