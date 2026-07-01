import { describe, expect, it } from 'vitest'
import {
  createOfficeTask,
  getCoworkerStatus,
  listCoworkerStatuses,
  listOfficeTasks,
  setCoworkerStatus,
  updateOfficeTask,
} from '../src/office/kanban.js'

describe('coworker statuses', () => {
  it('sets and reads a status case-insensitively', () => {
    setCoworkerStatus({ name: 'Alice', status: 'working', note: ' shipping ' })
    const status = getCoworkerStatus(' alice ')
    expect(status?.status).toBe('working')
    expect(status?.note).toBe('shipping')
  })

  it('drops empty notes', () => {
    setCoworkerStatus({ name: 'Bob', status: 'available', note: '   ' })
    expect(getCoworkerStatus('bob')?.note).toBeUndefined()
  })

  it('lists statuses sorted by name', () => {
    setCoworkerStatus({ name: 'Zed', status: 'away' })
    setCoworkerStatus({ name: 'Amy', status: 'available' })
    const names = listCoworkerStatuses().map(status => status.name)
    expect(names).toEqual([...names].sort((a, b) => a.localeCompare(b)))
  })
})

describe('office tasks', () => {
  it('creates a task with defaults', () => {
    const task = createOfficeTask({ title: '  Write docs  ', createdBy: 'alice' })
    expect(task.title).toBe('Write docs')
    expect(task.status).toBe('todo')
    expect(task.priority).toBe('normal')
    expect(task.description).toBeUndefined()
  })

  it('updates fields and preserves the rest', () => {
    const task = createOfficeTask({ title: 'Fix bug', createdBy: 'bob', description: 'details' })
    const updated = updateOfficeTask({ id: task.id, status: 'doing', assignee: ' Carol ' })
    expect(updated.status).toBe('doing')
    expect(updated.assignee).toBe('Carol')
    expect(updated.description).toBe('details')
    expect(updated.createdAt).toBe(task.createdAt)
  })

  it('clears description when updated to an empty string', () => {
    const task = createOfficeTask({ title: 'Cleanup', createdBy: 'bob', description: 'old' })
    const updated = updateOfficeTask({ id: task.id, description: '  ' })
    expect(updated.description).toBeUndefined()
  })

  it('throws for an unknown task id', () => {
    expect(() => updateOfficeTask({ id: 'missing', status: 'done' })).toThrow(/No office task/)
  })

  it('filters by status and assignee', () => {
    const task = createOfficeTask({ title: 'Filter me', createdBy: 'alice', assignee: 'Dana' })
    updateOfficeTask({ id: task.id, status: 'review' })
    const results = listOfficeTasks({ status: 'review', assignee: ' dana ' })
    expect(results.some(item => item.id === task.id)).toBe(true)
    expect(listOfficeTasks({ status: 'canceled', assignee: 'dana' }).some(item => item.id === task.id)).toBe(false)
  })
})
