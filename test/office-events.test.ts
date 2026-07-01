import { describe, expect, it } from 'vitest'
import { formatOfficeEvent, handleReasoningChunk } from '../src/cli/office-events.js'

describe('formatOfficeEvent', () => {
  it('formats direct-message signals', () => {
    const line = formatOfficeEvent('Alice', {
      type: 'data-signal',
      data: { attributes: { kind: 'direct-message' }, contents: 'bob says: hi' },
    })
    expect(line?.text).toContain('Alice received: bob says: hi')
  })

  it('formats send_office_message tool calls, with a broadcast marker for All', () => {
    const direct = formatOfficeEvent('Alice', { type: 'tool-call', payload: { toolName: 'send_office_message', args: { to: 'Bob', message: 'hi' } } })
    expect(direct?.text).toContain('Alice → Bob: hi')
    const broadcast = formatOfficeEvent('Alice', { type: 'tool-call', payload: { toolName: 'send_office_message', args: { to: 'All', message: 'hi' } } })
    expect(broadcast?.text).toContain('📣')
  })

  it('does not throw on tool events with missing args or result', () => {
    expect(formatOfficeEvent('Alice', { type: 'tool-call', payload: { toolName: 'send_office_message' } })).toBeDefined()
    expect(formatOfficeEvent('Alice', { type: 'tool-result', payload: { toolName: 'ask_human_question' } })).toBeDefined()
    expect(formatOfficeEvent('Alice', { type: 'tool-call' })).toBeDefined()
  })

  it('formats errors with a fallback message', () => {
    expect(formatOfficeEvent('Alice', { type: 'error', payload: {} })?.text).toContain('unknown error')
    expect(formatOfficeEvent('Alice', { type: 'error', payload: { error: { message: 'boom' } } })?.text).toContain('boom')
  })

  it('ignores unknown chunk types', () => {
    expect(formatOfficeEvent('Alice', { type: 'something-else' })).toBeUndefined()
    expect(formatOfficeEvent('Alice', {})).toBeUndefined()
  })
})

describe('handleReasoningChunk', () => {
  it('buffers deltas and emits one normalized reasoning line at the end', () => {
    const logged: string[] = []
    const addLog = (text: string) => logged.push(text)
    expect(handleReasoningChunk('Alice', { type: 'reasoning-start', payload: { id: 'r1' } }, addLog)).toBe(true)
    expect(handleReasoningChunk('Alice', { type: 'reasoning-delta', payload: { id: 'r1', text: 'thinking  about ' } }, addLog)).toBe(true)
    expect(handleReasoningChunk('Alice', { type: 'reasoning-delta', payload: { id: 'r1', text: 'jokes' } }, addLog)).toBe(true)
    expect(handleReasoningChunk('Alice', { type: 'reasoning-end', payload: { id: 'r1' } }, addLog)).toBe(true)
    expect(logged[0]).toContain('Alice is thinking')
    expect(logged[1]).toBe('💭 Alice reasoning: thinking about jokes')
  })

  it('returns false for non-reasoning chunks', () => {
    expect(handleReasoningChunk('Alice', { type: 'tool-call' }, () => {})).toBe(false)
  })

  it('emits nothing when the reasoning buffer is empty', () => {
    const logged: string[] = []
    const addLog = (text: string) => logged.push(text)
    handleReasoningChunk('Alice', { type: 'reasoning-start', payload: { id: 'r2' } }, addLog)
    handleReasoningChunk('Alice', { type: 'reasoning-end', payload: { id: 'r2' } }, addLog)
    expect(logged).toHaveLength(1)
  })
})
