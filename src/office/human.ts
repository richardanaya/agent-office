import type { Agent } from '@mastra/core/agent'
import { coworkers } from '../agents/coworkers.js'
import { mastra } from '../mastra/index.js'
import { officeMailbox } from './mailbox.js'
import { coworkerThread } from './threads.js'
import { listAgentCoworkers } from './coworkers.js'

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

function recentContextFor(coworkerName: string, limit = 12) {
  const coworker = coworkerName.trim().toLowerCase()
  const relevant = officeMailbox
    .list()
    .filter(message => {
      const from = message.from.toLowerCase()
      const to = message.to.toLowerCase()
      return from === coworker || to === coworker || to === 'all'
    })
    .slice(-limit)

  if (relevant.length === 0) return ''
  return relevant.map(message => `${message.from} → ${message.to}: ${message.body}`).join('\n')
}

function messageWithContext(coworkerName: string, currentBody: string, publicMessage = false) {
  const context = recentContextFor(coworkerName)
  const current = publicMessage ? `[Public office message to All] ${currentBody}` : currentBody
  if (!context) return current
  return `Recent office context for continuity only; do not re-answer old messages unless needed:\n${context}\n\nCurrent Human message:\n${current}`
}

export async function deliverHumanMessage(to: string, body: string) {
  if (to.trim().toLowerCase() === 'all') {
    const sent = officeMailbox.send({ from: HUMAN_NAME, to: 'all', body })
    officeMailbox.markDelivered(sent.id)
    for (const coworker of listAgentCoworkers()) {
      const agent = resolveCoworkerAgent(coworker.name)
      if (!agent) continue
      const result = agent.queueMessage(
        {
          contents: messageWithContext(coworker.name, body, true),
          attributes: { name: HUMAN_NAME, sentFrom: 'agent-office', broadcast: true, to: 'All' },
        },
        coworkerThread(coworker.name),
      )
      const accepted = await result.accepted
      if (accepted.action === 'discard') throw new Error(`Message to ${coworker.name} was discarded.`)
    }
    return sent
  }

  const sent = sendHumanMessage(to, body)
  // Human messages wake agents through sendMessage. Mark delivered immediately so
  // OfficeSignals does not also notify the same message and run the agent twice.
  officeMailbox.markDelivered(sent.id)

  const agent = resolveCoworkerAgent(to)
  if (!agent) throw new Error(`No coworker agent found for ${to}.`)

  const result = agent.queueMessage(
    { contents: messageWithContext(to, sent.body), attributes: { name: HUMAN_NAME, sentFrom: 'agent-office' } },
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