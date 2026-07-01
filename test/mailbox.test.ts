import { describe, expect, it } from 'vitest'
import { OfficeMailbox } from '../src/office/mailbox.js'
import { normalizeCoworkerName } from '../src/office/names.js'

describe('normalizeCoworkerName', () => {
  it('trims and lowercases', () => {
    expect(normalizeCoworkerName('  Alice ')).toBe('alice')
  })
})

describe('OfficeMailbox', () => {
  it('normalizes sender and recipient on send', () => {
    const mailbox = new OfficeMailbox()
    const sent = mailbox.send({ from: ' Alice ', to: 'BOB', body: 'hi' })
    expect(sent.from).toBe('alice')
    expect(sent.to).toBe('bob')
    expect(sent.deliveredAt).toBeUndefined()
  })

  it('filters messages by recipient regardless of casing', () => {
    const mailbox = new OfficeMailbox()
    mailbox.send({ from: 'alice', to: 'bob', body: 'one' })
    mailbox.send({ from: 'alice', to: 'carol', body: 'two' })
    expect(mailbox.messagesFor('Bob')).toHaveLength(1)
    expect(mailbox.messagesFor('Bob')[0]!.body).toBe('one')
  })

  it('tracks undelivered messages until marked delivered', () => {
    const mailbox = new OfficeMailbox()
    const sent = mailbox.send({ from: 'alice', to: 'bob', body: 'hi' })
    expect(mailbox.undeliveredFor('bob')).toHaveLength(1)
    mailbox.markDelivered(sent.id)
    expect(mailbox.undeliveredFor('bob')).toHaveLength(0)
    expect(mailbox.messagesFor('bob')).toHaveLength(1)
  })

  it('does not overwrite the delivery timestamp when marked twice', () => {
    const mailbox = new OfficeMailbox()
    const sent = mailbox.send({ from: 'alice', to: 'bob', body: 'hi' })
    mailbox.markDelivered(sent.id)
    const first = mailbox.messagesFor('bob')[0]!.deliveredAt
    mailbox.markDelivered(sent.id)
    expect(mailbox.messagesFor('bob')[0]!.deliveredAt).toBe(first)
  })

  it('list returns a copy, not internal state', () => {
    const mailbox = new OfficeMailbox()
    mailbox.send({ from: 'alice', to: 'bob', body: 'hi' })
    const listed = mailbox.list()
    listed.pop()
    expect(mailbox.list()).toHaveLength(1)
  })

  it('prunes oldest delivered messages past the cap but keeps undelivered ones', () => {
    const mailbox = new OfficeMailbox()
    const pending = mailbox.send({ from: 'alice', to: 'bob', body: 'keep me' })
    for (let i = 0; i < 2100; i++) {
      const sent = mailbox.send({ from: 'alice', to: 'bob', body: `msg ${i}` })
      mailbox.markDelivered(sent.id)
    }
    const remaining = mailbox.list()
    expect(remaining.length).toBeLessThanOrEqual(2000)
    expect(remaining.some(message => message.id === pending.id)).toBe(true)
  })
})
