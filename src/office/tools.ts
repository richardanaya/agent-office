import { createTool } from '@mastra/core/tools'
import { z } from 'zod'
import { officeMailbox } from './mailbox.js'
import { listAgentCoworkers, listCoworkerProfiles } from './coworkers.js'
import { clearScheduledAction, getScheduledAction, scheduleSelfWake } from './scheduled-actions.js'
import { createOfficeTask, listCoworkerStatuses, listOfficeTasks, setCoworkerStatus, updateOfficeTask } from './kanban.js'
import { askHumanQuestion } from './human-questions.js'

// Throttle window that stops an agent from double-messaging Human. This
// counteracts a real model behavior (re-sending or rephrasing an answer in a
// follow-up pass); keep it aligned with the "stop after answering" prompt rule.
const HUMAN_REPLY_THROTTLE_MS = 8_000
const recentHumanReplies = new Map<string, number>()

function pruneRecentHumanReplies(now: number) {
  for (const [key, sentAt] of recentHumanReplies) {
    if (now - sentAt >= HUMAN_REPLY_THROTTLE_MS) recentHumanReplies.delete(key)
  }
}

export function createSendOfficeMessageTool(from: string) {
  return createTool({
    id: 'send_office_message',
    description: 'Send a direct message to another virtual office coworker by name, to Human, or to "All" to broadcast to the whole office.',
    inputSchema: z.object({
      to: z.string().describe('The coworker name, Human, or All to broadcast to all agents and Human.'),
      message: z.string().describe('The message to send.'),
    }),
    outputSchema: z.object({
      id: z.string(),
      from: z.string(),
      to: z.string(),
      queued: z.boolean(),
      note: z.string(),
    }),
    execute: async ({ to, message }) => {
      const knownRecipients = ['all', ...listCoworkerProfiles().map(profile => profile.name.toLowerCase())]
      if (!knownRecipients.includes(to.trim().toLowerCase())) {
        throw new Error(
          `Unknown recipient "${to}". Valid recipients: ${[...listCoworkerProfiles().map(profile => profile.name), 'All'].join(', ')}. Use list_coworkers for the current directory.`,
        )
      }

      const pendingWake = getScheduledAction(from)
      if (to.trim().toLowerCase() === 'human' && pendingWake) {
        throw new Error(
          `You have a scheduled wake pending: ${JSON.stringify(pendingWake)}. Wait for that wake before sending a final message to Human, or use clear_scheduled_action first if the plan has changed.`,
        )
      }

      const normalizedTo = to.trim().toLowerCase()
      if (normalizedTo === 'human') {
        const key = `${from.toLowerCase()}:human`
        const now = Date.now()
        pruneRecentHumanReplies(now)
        const lastSentAt = recentHumanReplies.get(key)
        if (lastSentAt && now - lastSentAt < HUMAN_REPLY_THROTTLE_MS) {
          throw new Error('You already sent Human a message moments ago. Stop now and wait for a new Human message before sending another reply.')
        }
        recentHumanReplies.set(key, now)
      }

      if (normalizedTo === 'all') {
        const recipients = [...listAgentCoworkers().map(coworker => coworker.name), 'Human'].filter(
          recipient => recipient.toLowerCase() !== from.toLowerCase(),
        )
        const sentMessages = recipients.map(recipient => officeMailbox.send({
          from,
          to: recipient,
          body: recipient.toLowerCase() === 'human' ? message : `[Public office message to All from ${from}] ${message}`,
        }))
        return {
          id: sentMessages[0]?.id ?? `broadcast_${Date.now()}`,
          from,
          to: 'all',
          queued: true,
          note: `Broadcast queued to ${recipients.join(', ')}. Recipient signal providers will deliver it shortly.`,
        }
      }

      const sent = officeMailbox.send({ from, to, body: message })
      return {
        id: sent.id,
        from: sent.from,
        to: sent.to,
        queued: true,
        note: 'Message queued in the office mailbox. The recipient signal provider will deliver it shortly.',
      }
    },
  })
}

const scheduledActionSchema = z.object({
  id: z.string().describe('Unique id for the scheduled wake action.'),
  agentName: z.string().describe('The agent that owns and will receive this scheduled wake.'),
  instruction: z.string().describe('The instruction that will be sent back to the agent at wake time.'),
  scheduledAt: z.string().describe('ISO timestamp when the wake was scheduled.'),
  wakeAt: z.string().describe('ISO timestamp when the wake notification should be sent.'),
  delaySeconds: z.number().int().describe('Delay, in whole seconds, between scheduling and wake time.'),
})

export function createScheduleSelfWakeTool(agentName: string) {
  return createTool({
    id: 'schedule_self_wake',
    description:
      'Schedule exactly one future wakeful notification to yourself. Use this when you need to pause and continue later, for example waiting 10 seconds for coworkers to reply before checking the inbox. This tool fails if you already have a future wake scheduled; call get_scheduled_action to inspect it or clear_scheduled_action before replacing it.',
    strict: true,
    inputSchema: z.object({
      delaySeconds: z
        .number()
        .int('delaySeconds must be a whole number of seconds, not a decimal.')
        .min(1, 'delaySeconds must be at least 1 second.')
        .max(3600, 'delaySeconds cannot be more than 3600 seconds / 1 hour.')
        .describe(
          'Required. Whole number of seconds from now before waking yourself. Must be 1 through 3600. Use small values like 10-30 seconds for short waits in demos.',
        ),
      instruction: z
        .string()
        .trim()
        .min(5, 'instruction must describe what to do when waking up.')
        .max(1000, 'instruction must be 1000 characters or less.')
        .describe(
          'Required. Clear imperative reminder for your future self, e.g. "Check whether Bob and Carol replied, then summarize the best joke to Human." Must be 5-1000 non-whitespace characters.',
        ),
    }),
    outputSchema: scheduledActionSchema,
    execute: async ({ delaySeconds, instruction }) => scheduleSelfWake({ agentName, delaySeconds, instruction }),
  })
}

export function createClearScheduledActionTool(agentName: string) {
  return createTool({
    id: 'clear_scheduled_action',
    description: 'Clear your currently scheduled self-wake action, if any.',
    inputSchema: z.object({}),
    strict: true,
    outputSchema: z.object({
      cleared: z.boolean().describe('True when a scheduled wake existed and was cleared.'),
      action: scheduledActionSchema.optional().describe('The cleared scheduled wake action, if one existed.'),
    }),
    execute: async () => {
      const action = clearScheduledAction(agentName)
      return { cleared: Boolean(action), action }
    },
  })
}

export function createGetScheduledActionTool(agentName: string) {
  return createTool({
    id: 'get_scheduled_action',
    description: 'Check whether you currently have a scheduled self-wake action.',
    inputSchema: z.object({}),
    strict: true,
    outputSchema: z.object({
      action: scheduledActionSchema.optional().describe('Current scheduled wake action, if one exists.'),
    }),
    execute: async () => ({ action: getScheduledAction(agentName) }),
  })
}

export const getCurrentTimeTool = createTool({
  id: 'get_current_time',
  description: 'Get the current time as Unix time in seconds. Use this when you need an exact current timestamp for scheduling or time calculations.',
  inputSchema: z.object({}),
  outputSchema: z.object({
    unixTime: z.number().int().describe('Current Unix timestamp in seconds.'),
  }),
  execute: async () => ({ unixTime: Math.floor(Date.now() / 1000) }),
})

export const convertUnixTimeToIso8601Tool = createTool({
  id: 'convert_unix_time_to_iso8601',
  description: 'Convert a Unix timestamp to an ISO 8601 UTC timestamp string.',
  strict: true,
  inputSchema: z.object({
    unixTime: z
      .number()
      .int('unixTime must be a whole number.')
      .min(0, 'unixTime must be non-negative.')
      .describe('Unix timestamp to convert. Seconds by default, or milliseconds when unit is "milliseconds".'),
    unit: z
      .enum(['seconds', 'milliseconds'])
      .default('seconds')
      .describe('Whether unixTime is in seconds or milliseconds. Defaults to seconds.'),
  }),
  outputSchema: z.object({
    iso8601: z.string().describe('ISO 8601 UTC timestamp.'),
  }),
  execute: async ({ unixTime, unit }) => {
    const milliseconds = unit === 'milliseconds' ? unixTime : unixTime * 1000
    const date = new Date(milliseconds)
    if (Number.isNaN(date.getTime())) throw new Error('Invalid Unix timestamp.')
    return { iso8601: date.toISOString() }
  },
})

export const listCoworkersTool = createTool({
  id: 'list_coworkers',
  description: 'List the current runtime coworkers in the virtual office and their roles. Use this for any question about coworker names, roles, availability, or who is in the office because the list can change at runtime.',
  inputSchema: z.object({}),
  outputSchema: z.object({
    coworkers: z.array(z.object({ name: z.string(), role: z.string() })),
  }),
  execute: async () => ({ coworkers: listCoworkerProfiles() }),
})

export function createAskHumanQuestionTool(agentName: string) {
  return createTool({
    id: 'ask_human_question',
    description: 'Ask Human one short-form question with single-choice or multiple-choice options. Human can always add a custom freeform answer. The question appears in the terminal UI and the answer is sent back as a Human message.',
    strict: true,
    inputSchema: z.object({
      prompt: z.string().trim().min(5).max(300),
      mode: z.enum(['single', 'multiple']).default('single'),
      choices: z.array(z.string().trim().min(1).max(120)).min(1).max(9),
      allowCustomAnswer: z.boolean().default(true),
    }),
    outputSchema: z.object({
      id: z.string(),
      from: z.string(),
      prompt: z.string(),
      mode: z.string(),
      choices: z.array(z.string()),
      allowCustomAnswer: z.boolean(),
      createdAt: z.string(),
    }),
    execute: async input => askHumanQuestion({ ...input, mode: input.mode ?? 'single', from: agentName }),
  })
}

export function createSetStatusTool(agentName: string) {
  return createTool({
    id: 'set_status',
    description: 'Set your visible office status so coworkers and Human can see what you are doing.',
    strict: true,
    inputSchema: z.object({
      status: z.enum(['available', 'thinking', 'working', 'waiting', 'blocked', 'done', 'away']),
      note: z.string().trim().max(200).optional().describe('Short optional status note.'),
    }),
    outputSchema: z.object({
      name: z.string(),
      status: z.string(),
      note: z.string().optional(),
      updatedAt: z.string(),
    }),
    execute: async ({ status, note }) => setCoworkerStatus({ name: agentName, status, note }),
  })
}

export const listStatusesTool = createTool({
  id: 'list_statuses',
  description: 'List visible coworker statuses.',
  inputSchema: z.object({}),
  outputSchema: z.object({
    statuses: z.array(z.object({ name: z.string(), status: z.string(), note: z.string().optional(), updatedAt: z.string() })),
  }),
  execute: async () => ({ statuses: listCoworkerStatuses() }),
})

export function createOfficeTaskTool(agentName: string) {
  return createTool({
    id: 'create_office_task',
    description: 'Create a task on the shared mini office Kanban board.',
    strict: true,
    inputSchema: z.object({
      title: z.string().trim().min(3).max(120),
      description: z.string().trim().max(1000).optional(),
      assignee: z.string().trim().max(64).optional(),
      priority: z.enum(['low', 'normal', 'high', 'urgent']).default('normal'),
    }),
    outputSchema: z.object({
      id: z.string(), title: z.string(), description: z.string().optional(), status: z.string(), assignee: z.string().optional(), priority: z.string(), createdBy: z.string(), createdAt: z.string(), updatedAt: z.string(),
    }),
    execute: async input => createOfficeTask({ ...input, createdBy: agentName }),
  })
}

export const updateOfficeTaskTool = createTool({
  id: 'update_office_task',
  description: 'Update a task on the shared mini office Kanban board: move columns, assign, reprioritize, or revise details.',
  strict: true,
  inputSchema: z.object({
    id: z.string(),
    title: z.string().trim().min(3).max(120).optional(),
    description: z.string().trim().max(1000).optional(),
    status: z.enum(['backlog', 'todo', 'doing', 'blocked', 'review', 'done', 'canceled']).optional(),
    assignee: z.string().trim().max(64).optional(),
    priority: z.enum(['low', 'normal', 'high', 'urgent']).optional(),
  }),
  outputSchema: z.object({
    id: z.string(), title: z.string(), description: z.string().optional(), status: z.string(), assignee: z.string().optional(), priority: z.string(), createdBy: z.string(), createdAt: z.string(), updatedAt: z.string(),
  }),
  execute: async input => updateOfficeTask(input),
})

export const listOfficeTasksTool = createTool({
  id: 'list_office_tasks',
  description: 'List tasks on the shared mini office Kanban board, optionally filtered by status or assignee.',
  inputSchema: z.object({
    status: z.enum(['backlog', 'todo', 'doing', 'blocked', 'review', 'done', 'canceled']).optional(),
    assignee: z.string().optional(),
  }),
  outputSchema: z.object({
    tasks: z.array(z.object({ id: z.string(), title: z.string(), description: z.string().optional(), status: z.string(), assignee: z.string().optional(), priority: z.string(), createdBy: z.string(), createdAt: z.string(), updatedAt: z.string() })),
  }),
  execute: async filter => ({ tasks: listOfficeTasks(filter) }),
})

export const listOfficeMessagesTool = createTool({
  id: 'list_office_messages',
  description: 'Inspect all messages in the virtual office mailbox. Useful for demos and debugging.',
  inputSchema: z.object({}),
  outputSchema: z.object({
    messages: z.array(
      z.object({
        id: z.string(),
        from: z.string(),
        to: z.string(),
        body: z.string(),
        createdAt: z.string(),
        deliveredAt: z.string().optional(),
      }),
    ),
  }),
  execute: async () => ({ messages: officeMailbox.list() }),
})
