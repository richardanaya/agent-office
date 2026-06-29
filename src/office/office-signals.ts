import { SignalProvider } from '@mastra/core/signals'
import type { SignalProviderTarget, SignalSubscription } from '@mastra/core/signals'
import { normalizeCoworkerName, officeMailbox } from './mailbox.js'

export class OfficeSignals extends SignalProvider<'office-signals'> {
  readonly id = 'office-signals' as const
  readonly name = 'Agent Office Signals'
  readonly pollInterval = 2_000

  watchCoworker(target: SignalProviderTarget, coworkerName: string): SignalSubscription {
    return this.subscribe(target, coworkerName.toLowerCase(), { coworkerName })
  }

  unwatchCoworker(target: SignalProviderTarget, coworkerName: string): boolean {
    return this.unsubscribe(target, coworkerName.toLowerCase())
  }

  async poll(subscriptions: SignalSubscription[]): Promise<void> {
    for (const sub of subscriptions) {
      const messages = officeMailbox.undeliveredFor(sub.externalResourceId)

      for (const message of messages) {
        if (normalizeCoworkerName(message.from) === 'human') continue

        await this.notify(
          {
            source: this.id,
            kind: 'direct-message',
            priority: 'medium',
            summary: `${message.from} says: ${message.body}`,
            payload: message,
            dedupeKey: `${this.id}:${message.id}`,
          },
          { resourceId: sub.resourceId, threadId: sub.threadId },
        )

        officeMailbox.markDelivered(message.id)
      }
    }
  }
}
