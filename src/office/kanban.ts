export type CoworkerStatus = {
  name: string
  status: 'available' | 'thinking' | 'working' | 'waiting' | 'blocked' | 'done' | 'away'
  note?: string
  updatedAt: string
}

export type OfficeTaskStatus = 'backlog' | 'todo' | 'doing' | 'blocked' | 'review' | 'done' | 'canceled'
export type OfficeTaskPriority = 'low' | 'normal' | 'high' | 'urgent'

export type OfficeTask = {
  id: string
  title: string
  description?: string
  status: OfficeTaskStatus
  assignee?: string
  priority: OfficeTaskPriority
  createdBy: string
  createdAt: string
  updatedAt: string
}

const statuses = new Map<string, CoworkerStatus>()
const tasks = new Map<string, OfficeTask>()

export function setCoworkerStatus(input: { name: string; status: CoworkerStatus['status']; note?: string }): CoworkerStatus {
  const status: CoworkerStatus = {
    name: input.name.trim(),
    status: input.status,
    note: input.note?.trim() || undefined,
    updatedAt: new Date().toISOString(),
  }
  statuses.set(status.name.toLowerCase(), status)
  return status
}

export function getCoworkerStatus(name: string): CoworkerStatus | undefined {
  return statuses.get(name.trim().toLowerCase())
}

export function listCoworkerStatuses(): CoworkerStatus[] {
  return [...statuses.values()].sort((a, b) => a.name.localeCompare(b.name))
}

export function createOfficeTask(input: {
  title: string
  description?: string
  assignee?: string
  priority?: OfficeTaskPriority
  createdBy: string
}): OfficeTask {
  const now = new Date().toISOString()
  const task: OfficeTask = {
    id: `task_${Date.now()}_${Math.random().toString(36).slice(2)}`,
    title: input.title.trim(),
    description: input.description?.trim() || undefined,
    assignee: input.assignee?.trim() || undefined,
    priority: input.priority ?? 'normal',
    status: 'todo',
    createdBy: input.createdBy,
    createdAt: now,
    updatedAt: now,
  }
  tasks.set(task.id, task)
  return task
}

export function updateOfficeTask(input: {
  id: string
  title?: string
  description?: string
  status?: OfficeTaskStatus
  assignee?: string
  priority?: OfficeTaskPriority
}): OfficeTask {
  const existing = tasks.get(input.id)
  if (!existing) throw new Error(`No office task found with id ${input.id}.`)
  const updated: OfficeTask = {
    ...existing,
    title: input.title === undefined ? existing.title : input.title.trim(),
    description: input.description === undefined ? existing.description : input.description.trim() || undefined,
    assignee: input.assignee === undefined ? existing.assignee : input.assignee.trim() || undefined,
    priority: input.priority ?? existing.priority,
    status: input.status ?? existing.status,
    updatedAt: new Date().toISOString(),
  }
  tasks.set(updated.id, updated)
  return updated
}

export function listOfficeTasks(filter: { status?: OfficeTaskStatus; assignee?: string } = {}): OfficeTask[] {
  return [...tasks.values()]
    .filter(task => !filter.status || task.status === filter.status)
    .filter(task => !filter.assignee || task.assignee?.toLowerCase() === filter.assignee.trim().toLowerCase())
    .sort((a, b) => a.createdAt.localeCompare(b.createdAt))
}
