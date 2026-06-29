import type { Agent } from '@mastra/core/agent'
import { coworkers } from '../agents/coworkers.js'
import { mastra } from '../mastra/index.js'
import { officeMailbox } from './mailbox.js'
import { coworkerThread } from './threads.js'

export const HUMAN_NAME = 'Human'

function resolveCoworkerAgent(name: string): Agent | undefined {
  const id = name.trim().toLowerCase()
  if (id in coworkers) return coworkers[id as keyof typeof coworkers]
  try {
    return mastra.getAgentById(id)
  } catch {
    return undefined
  }
}

export function sendHumanMessage(to: string, body: string) {
  return officeMailbox.send({ from: HUMAN_NAME, to, body })
}

export async function deliverHumanMessage(to: string, body: string) {
  const sent = sendHumanMessage(to, body)
  // Human messages wake agents through sendMessage. Mark delivered immediately so
  // OfficeSignals does not also notify the same message and run the agent twice.
  officeMailbox.markDelivered(sent.id)

  const agent = resolveCoworkerAgent(to)
  if (!agent) throw new Error(`No coworker agent found for ${to}.`)

  const result = agent.queueMessage(
    { contents: sent.body, attributes: { name: HUMAN_NAME, sentFrom: 'agent-office' } },
    coworkerThread(to),
  )

  const accepted = await result.accepted
  if (accepted.action === 'discard') throw new Error(`Message to ${to} was discarded.`)

  return sent
}

export function listHumanInbox(options: { unreadOnly?: boolean } = {}) {
  const messages = options.unreadOnly ? officeMailbox.undeliveredFor(HUMAN_NAME) : officeMailbox.messagesFor(HUMAN_NAME)
  return messages
}

export function markHumanMessageSeen(id: string) {
  officeMailbox.markDelivered(id)
}

export function markAllHumanMessagesSeen() {
  for (const message of officeMailbox.undeliveredFor(HUMAN_NAME)) {
    officeMailbox.markDelivered(message.id)
  }
}