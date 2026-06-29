export type OfficeMessage = {
  id: string
  from: string
  to: string
  body: string
  createdAt: string
  deliveredAt?: string
}

export class OfficeMailbox {
  private messages: OfficeMessage[] = []

  send(input: { from: string; to: string; body: string }): OfficeMessage {
    const message: OfficeMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).slice(2)}`,
      from: normalizeCoworkerName(input.from),
      to: normalizeCoworkerName(input.to),
      body: input.body,
      createdAt: new Date().toISOString(),
    }

    this.messages.push(message)
    return message
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

export function normalizeCoworkerName(name: string): string {
  return name.trim().toLowerCase()
}

export const officeMailbox = new OfficeMailbox()
