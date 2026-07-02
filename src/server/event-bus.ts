import type { OfficeEvent, StoredOfficeEvent } from '../protocol.js'

const EVENT_BUFFER_LIMIT = 200

// Fan-out bus with a replay buffer so clients can reconnect with ?since=<id>
// and miss nothing (up to the buffer limit).
export class OfficeEventBus {
  private nextId = 1
  private buffer: StoredOfficeEvent[] = []
  private subscribers = new Set<(event: StoredOfficeEvent) => void>()

  publish(event: OfficeEvent): StoredOfficeEvent {
    const stored: StoredOfficeEvent = { id: this.nextId++, event }
    this.buffer.push(stored)
    if (this.buffer.length > EVENT_BUFFER_LIMIT) this.buffer.splice(0, this.buffer.length - EVENT_BUFFER_LIMIT)
    for (const subscriber of this.subscribers) {
      try {
        subscriber(stored)
      } catch {
        // One broken subscriber must not stop delivery to the others.
      }
    }
    return stored
  }

  eventsSince(id: number): StoredOfficeEvent[] {
    return this.buffer.filter(stored => stored.id > id)
  }

  subscribe(subscriber: (event: StoredOfficeEvent) => void): () => void {
    this.subscribers.add(subscriber)
    return () => {
      this.subscribers.delete(subscriber)
    }
  }
}
