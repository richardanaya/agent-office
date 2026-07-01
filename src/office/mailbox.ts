import { createId } from './ids.js'
import { normalizeCoworkerName } from './names.js'

export type OfficeMessage = {
  id: string
  from: string
  to: string
  body: string
  createdAt: string
  deliveredAt?: string
}

const MAX_MAILBOX_MESSAGES = 2000

export class OfficeMailbox {
  private messages: OfficeMessage[] = []

  send(input: { from: string; to: string; body: string }): OfficeMessage {
    const message: OfficeMessage = {
      id: createId('msg'),
      from: normalizeCoworkerName(input.from),
      to: normalizeCoworkerName(input.to),
      body: input.body,
      createdAt: new Date().toISOString(),
    }

    this.messages.push(message)
    this.prune()
    return message
  }

  // Drop the oldest delivered messages once the mailbox exceeds its cap;
  // undelivered messages are never dropped.
  private prune(): void {
    let excess = this.messages.length - MAX_MAILBOX_MESSAGES
    if (excess <= 0) return
    this.messages = this.messages.filter(message => {
      if (excess > 0 && message.deliveredAt) {
        excess--
        return false
      }
      return true
    })
  }

  messagesFor(coworkerName: string): OfficeMessage[] {
    const to = normalizeCoworkerName(coworkerName)
    return this.messages.filter(message => message.to === to)
  }

  undeliveredFor(coworkerName: string): OfficeMessage[] {
    return this.messagesFor(coworkerName).filter(message => !message.deliveredAt)
  }

  markDelivered(id: string): void {
    const message = this.messages.find(message => message.id === id)
    if (message && !message.deliveredAt) message.deliveredAt = new Date().toISOString()
  }

  list(): OfficeMessage[] {
    return [...this.messages]
  }
}

export const officeMailbox = new OfficeMailbox()
