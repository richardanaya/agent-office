import { createId } from './ids.js'
import { officeMailbox } from './mailbox.js'
import { normalizeCoworkerName } from './names.js'

export type ScheduledAction = {
  id: string
  agentName: string
  instruction: string
  scheduledAt: string
  wakeAt: string
  delaySeconds: number
}

type ScheduledActionRecord = ScheduledAction & { timeout: NodeJS.Timeout }

const scheduledActions = new Map<string, ScheduledActionRecord>()

export function getScheduledAction(agentName: string): ScheduledAction | undefined {
  const record = scheduledActions.get(normalizeCoworkerName(agentName))
  if (!record) return undefined
  const { timeout: _timeout, ...action } = record
  return action
}

export function scheduleSelfWake(input: { agentName: string; instruction: string; delaySeconds: number }): ScheduledAction {
  const key = normalizeCoworkerName(input.agentName)
  const instruction = input.instruction.trim()

  if (!Number.isInteger(input.delaySeconds) || input.delaySeconds < 1 || input.delaySeconds > 3600) {
    throw new Error('delaySeconds must be an integer from 1 to 3600.')
  }

  if (instruction.length < 5 || instruction.length > 1000) {
    throw new Error('instruction must be between 5 and 1000 characters after trimming whitespace.')
  }

  const existing = getScheduledAction(input.agentName)
  if (existing) {
    throw new Error(
      `You are already waiting on a future item: ${JSON.stringify(existing)}. Do not schedule another wake. Do not clear or replace this wake unless the Human explicitly changed the plan. Wait for the existing wake to fire.`,
    )
  }

  const delaySeconds = input.delaySeconds
  const now = Date.now()
  const action: ScheduledAction = {
    id: createId('wake'),
    agentName: input.agentName,
    instruction,
    scheduledAt: new Date(now).toISOString(),
    wakeAt: new Date(now + delaySeconds * 1_000).toISOString(),
    delaySeconds,
  }

  const timeout = setTimeout(() => {
    officeMailbox.send({
      from: 'scheduler',
      to: input.agentName,
      body: `Scheduled self-wake reminder: ${instruction}`,
    })
    scheduledActions.delete(key)
  }, delaySeconds * 1_000)

  scheduledActions.set(key, { ...action, timeout })
  return action
}

export function clearScheduledAction(agentName: string): ScheduledAction | undefined {
  const key = normalizeCoworkerName(agentName)
  const record = scheduledActions.get(key)
  if (!record) return undefined
  clearTimeout(record.timeout)
  scheduledActions.delete(key)
  const { timeout: _timeout, ...action } = record
  return action
}
