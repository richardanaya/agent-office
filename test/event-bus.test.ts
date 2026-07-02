import { describe, expect, it } from 'vitest'
import { OfficeEventBus } from '../src/server/event-bus.js'
import type { OfficeEvent } from '../src/protocol.js'

function messageEvent(body: string): OfficeEvent {
  return { type: 'human-message-sent', to: 'All', body }
}

describe('OfficeEventBus', () => {
  it('assigns increasing ids and notifies subscribers', () => {
    const bus = new OfficeEventBus()
    const seen: number[] = []
    bus.subscribe(stored => seen.push(stored.id))
    const first = bus.publish(messageEvent('one'))
    const second = bus.publish(messageEvent('two'))
    expect(second.id).toBeGreaterThan(first.id)
    expect(seen).toEqual([first.id, second.id])
  })

  it('replays only events after the given id', () => {
    const bus = new OfficeEventBus()
    bus.publish(messageEvent('one'))
    const second = bus.publish(messageEvent('two'))
    const third = bus.publish(messageEvent('three'))
    expect(bus.eventsSince(second.id).map(stored => stored.id)).toEqual([third.id])
    expect(bus.eventsSince(0)).toHaveLength(3)
  })

  it('unsubscribes cleanly', () => {
    const bus = new OfficeEventBus()
    const seen: number[] = []
    const unsubscribe = bus.subscribe(stored => seen.push(stored.id))
    bus.publish(messageEvent('one'))
    unsubscribe()
    bus.publish(messageEvent('two'))
    expect(seen).toHaveLength(1)
  })

  it('keeps delivering when one subscriber throws', () => {
    const bus = new OfficeEventBus()
    const seen: number[] = []
    bus.subscribe(() => {
      throw new Error('boom')
    })
    bus.subscribe(stored => seen.push(stored.id))
    bus.publish(messageEvent('one'))
    expect(seen).toHaveLength(1)
  })

  it('caps the replay buffer', () => {
    const bus = new OfficeEventBus()
    for (let index = 0; index < 250; index++) bus.publish(messageEvent(`msg ${index}`))
    const buffered = bus.eventsSince(0)
    expect(buffered.length).toBeLessThanOrEqual(200)
    expect(buffered.at(-1)!.id).toBe(250)
  })
})
