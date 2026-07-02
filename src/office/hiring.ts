import type { Agent } from '@mastra/core/agent'
import {
  createCoworkerAgent,
  registerCoworkerAgent,
  subscribeCoworkerThread,
  unregisterCoworkerAgent,
  unsubscribeCoworkerThread,
} from '../agents/coworkers.js'
import { mastra } from '../mastra/index.js'
import { addAgentCoworker, listAgentCoworkers, removeAgentCoworker, type CoworkerProfile } from './coworkers.js'
import { officeMailbox } from './mailbox.js'
import { normalizeCoworkerName } from './names.js'

export type HiredCoworker = { profile: CoworkerProfile; agent: Agent }

// Everything a coworker needs to exist in the office: directory profile,
// agent instance, Mastra registration, and signal subscription. With
// announce, existing coworkers get a directory-update message (skip it for
// bulk team loads to avoid waking every agent at startup).
export function hireCoworker(input: CoworkerProfile, options: { announce?: boolean } = {}): HiredCoworker {
  const existing = listAgentCoworkers().map(profile => profile.name)
  const profile = addAgentCoworker(input)
  const agent = createCoworkerAgent(profile.name, profile.role)
  registerCoworkerAgent(profile.name, agent)
  mastra.addAgent(agent, normalizeCoworkerName(profile.name))
  subscribeCoworkerThread(profile.name)
  if (options.announce) {
    for (const coworker of existing) {
      officeMailbox.send({
        from: 'office-admin',
        to: coworker,
        body: `Coworker directory updated: ${profile.name} joined as ${profile.role}. Use list_coworkers for the current full list.`,
      })
    }
  }
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
