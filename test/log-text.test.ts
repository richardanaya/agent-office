import { describe, expect, it } from 'vitest'
import { expandLogLines, logContentWidth, normalizeLogText, wrapLogText } from '../src/cli/log-text.js'

describe('normalizeLogText', () => {
  it('collapses whitespace runs and trims', () => {
    expect(normalizeLogText('  a\n\n b\t c ')).toBe('a b c')
  })
})

describe('logContentWidth', () => {
  it('never goes below the minimum width', () => {
    expect(logContentWidth(10)).toBe(20)
    expect(logContentWidth(100)).toBe(88)
  })
})

describe('wrapLogText', () => {
  it('returns the text untouched when it fits', () => {
    expect(wrapLogText('short', 20)).toEqual(['short'])
  })

  it('wraps on word boundaries', () => {
    expect(wrapLogText('alpha beta gamma', 11)).toEqual(['alpha beta', 'gamma'])
  })

  it('splits words longer than the width', () => {
    expect(wrapLogText('abcdefghij', 4)).toEqual(['abcd', 'efgh', 'ij'])
  })

  it('preserves blank lines and handles CRLF', () => {
    expect(wrapLogText('one\r\n\r\ntwo', 20)).toEqual(['one', '', 'two'])
  })

  it('returns a single empty line for empty input', () => {
    expect(wrapLogText('', 20)).toEqual([''])
  })

  it('accounts for wide characters', () => {
    // Each CJK character is two columns wide, so only two fit per line.
    expect(wrapLogText('你好世界', 4)).toEqual(['你好', '世界'])
  })
})

describe('expandLogLines', () => {
  it('gives each wrapped segment a stable, unique key', () => {
    const lines = expandLogLines([{ id: 7, text: 'alpha beta gamma', color: 'red' }], 11)
    expect(lines).toEqual([
      { key: '7:0', text: 'alpha beta', color: 'red' },
      { key: '7:1', text: 'gamma', color: 'red' },
    ])
  })
})
