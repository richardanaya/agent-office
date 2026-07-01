import { describe, expect, it } from 'vitest'
import { parseMessageTarget } from '../src/cli/message-target.js'

const targets = ['All', 'Alice', 'Bob']

describe('parseMessageTarget', () => {
  it('sends plain text to the default target', () => {
    expect(parseMessageTarget('hello everyone', targets, 'All')).toEqual({ to: 'All', body: 'hello everyone', mentioned: false })
  })

  it('routes @name prefixes to the named coworker, case-insensitively', () => {
    expect(parseMessageTarget('@bob how is it going?', targets, 'All')).toEqual({ to: 'Bob', body: 'how is it going?', mentioned: true })
    expect(parseMessageTarget('@ALICE hi', targets, 'All')).toEqual({ to: 'Alice', body: 'hi', mentioned: true })
  })

  it('supports @all to broadcast when pinned elsewhere', () => {
    expect(parseMessageTarget('@all standup time', targets, 'Bob')).toEqual({ to: 'All', body: 'standup time', mentioned: true })
  })

  it('treats unknown mentions as plain text to the default target', () => {
    expect(parseMessageTarget('@nobody hello', targets, 'Alice')).toEqual({ to: 'Alice', body: '@nobody hello', mentioned: false })
  })

  it('does not treat mid-message @ as a mention', () => {
    expect(parseMessageTarget('email me @bob.com', targets, 'All')).toEqual({ to: 'All', body: 'email me @bob.com', mentioned: false })
  })

  it('handles a mention with no body', () => {
    expect(parseMessageTarget('@bob', targets, 'All')).toEqual({ to: 'Bob', body: '', mentioned: true })
  })
})
