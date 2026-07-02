import { afterEach, describe, expect, it } from 'vitest'
import { deleteWikiPage, listWikiPages, readWikiPage, replaceWikiPages, wikiSlug, writeWikiPage } from '../src/office/wiki.js'

afterEach(() => {
  replaceWikiPages([])
})

describe('wikiSlug', () => {
  it('kebab-cases titles', () => {
    expect(wikiSlug('  Launch Plan: Q3!  ')).toBe('launch-plan-q3')
    expect(wikiSlug('API Notes')).toBe('api-notes')
  })
})

describe('office wiki', () => {
  it('creates and reads a page by title or slug', () => {
    const page = writeWikiPage({ title: 'Launch Plan', content: 'Ship it.', updatedBy: 'Alice' })
    expect(page.slug).toBe('launch-plan')
    expect(readWikiPage('Launch Plan')?.content).toBe('Ship it.')
    expect(readWikiPage('launch-plan')?.updatedBy).toBe('Alice')
  })

  it('overwrites by title and keeps createdAt', () => {
    const first = writeWikiPage({ title: 'Notes', content: 'v1', updatedBy: 'Alice' })
    const second = writeWikiPage({ title: 'notes', content: 'v2', updatedBy: 'Bob' })
    expect(second.slug).toBe(first.slug)
    expect(second.createdAt).toBe(first.createdAt)
    expect(readWikiPage('Notes')?.content).toBe('v2')
    expect(readWikiPage('Notes')?.updatedBy).toBe('Bob')
    expect(listWikiPages()).toHaveLength(1)
  })

  it('validates titles and content', () => {
    expect(() => writeWikiPage({ title: 'x', content: 'ok', updatedBy: 'a' })).toThrow(/2-120/)
    expect(() => writeWikiPage({ title: '!!!', content: 'ok', updatedBy: 'a' })).toThrow(/letter or number/)
    expect(() => writeWikiPage({ title: 'Valid', content: '', updatedBy: 'a' })).toThrow(/content/)
  })

  it('lists pages sorted by title and deletes by title', () => {
    writeWikiPage({ title: 'Zebra', content: 'z', updatedBy: 'a' })
    writeWikiPage({ title: 'Apple', content: 'a', updatedBy: 'a' })
    expect(listWikiPages().map(page => page.title)).toEqual(['Apple', 'Zebra'])
    expect(deleteWikiPage('zebra')?.title).toBe('Zebra')
    expect(deleteWikiPage('zebra')).toBeUndefined()
    expect(listWikiPages()).toHaveLength(1)
  })

  it('replaceWikiPages swaps the whole wiki', () => {
    writeWikiPage({ title: 'Old', content: 'old', updatedBy: 'a' })
    const now = new Date().toISOString()
    replaceWikiPages([{ slug: 'new-page', title: 'New Page', content: 'new', updatedBy: 'b', createdAt: now, updatedAt: now }])
    expect(listWikiPages().map(page => page.slug)).toEqual(['new-page'])
  })
})
