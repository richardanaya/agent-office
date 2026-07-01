import type { Agent } from '@mastra/core/agent'
import {
  createCoworkerAgent,
  registerCoworkerAgent,
  subscribeCoworkerThread,
  unregisterCoworkerAgent,
  unsubscribeCoworkerThread,
} from '../agents/coworkers.js'
import { mastra } from '../mastra/index.js'
import { addAgentCoworker, removeAgentCoworker, type CoworkerProfile } from './coworkers.js'
import { normalizeCoworkerName } from './names.js'

export type HiredCoworker = { profile: CoworkerProfile; agent: Agent }

// Everything a coworker needs to exist in the office: directory profile,
// agent instance, Mastra registration, and signal subscription.
export function hireCoworker(input: CoworkerProfile): HiredCoworker {
  const profile = addAgentCoworker(input)
  const agent = createCoworkerAgent(profile.name, profile.role)
  registerCoworkerAgent(profile.name, agent)
  mastra.addAgent(agent, normalizeCoworkerName(profile.name))
  subscribeCoworkerThread(profile.name)
  return { profile, agent }
}

export function fireCoworker(name: string): CoworkerProfile | undefined {
  const removed = removeAgentCoworker(name)
  if (!removed) return undefined
  unsubscribeCoworkerThread(removed.name)
  unregisterCoworkerAgent(removed.name)
  mastra.removeAgent(normalizeCoworkerName(removed.name))
  return removed
}
