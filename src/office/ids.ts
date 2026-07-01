import { randomUUID } from 'node:crypto'

// Timestamp-prefixed so ids sort roughly by creation time.
export function createId(prefix: string): string {
  return `${prefix}_${Date.now()}_${randomUUID().slice(0, 8)}`
}
