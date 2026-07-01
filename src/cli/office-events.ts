import { normalizeLogText } from './log-text.js'

export type Chunk = { type?: string; payload?: any; data?: any }
export type OfficeEventLine = { text: string; color?: string }

const reasoningBuffers = new Map<string, string>()
const activeReasoning = new Set<string>()

export function handleReasoningChunk(agentName: string, chunk: Chunk, addLog: (text: string, color?: string) => void) {
  switch (chunk.type) {
    case 'reasoning-start': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      reasoningBuffers.set(key, '')
      if (activeReasoning.has(key)) return true
      activeReasoning.add(key)
      addLog(`💭 ${agentName} is thinking…`, 'gray')
      return true
    }
    case 'reasoning-delta': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      const delta = chunk.payload?.text ?? ''
      reasoningBuffers.set(key, `${reasoningBuffers.get(key) ?? ''}${delta}`)
      return true
    }
    case 'reasoning-end': {
      const id = chunk.payload?.id ?? 'default'
      const key = `${agentName}:${id}`
      activeReasoning.delete(key)
      const reasoning = normalizeLogText(reasoningBuffers.get(key) ?? '')
      reasoningBuffers.delete(key)
      if (reasoning) addLog(`💭 ${agentName} reasoning: ${reasoning}`, 'gray')
      return true
    }
    default:
      return false
  }
}

export function formatOfficeEvent(agentName: string, chunk: Chunk): OfficeEventLine | undefined {
  switch (chunk.type) {
    case 'data-signal': {
      const attrs = chunk.data?.attributes ?? {}
      const contents = chunk.data?.contents
      if (attrs.kind === 'direct-message' && contents) return { text: `📨 ${agentName} received: ${contents}`, color: 'yellow' }
      return
    }
    case 'tool-call': {
      const { toolName, args = {} } = chunk.payload ?? {}
      if (toolName === 'send_office_message') return { text: `${args.to?.toLowerCase?.() === 'all' ? '📣' : '✉️ '} ${agentName} → ${args.to}: ${args.message}`, color: 'magenta' }
      if (toolName === 'ask_human_question') return { text: `❓ ${agentName} asked Human: ${args.prompt}`, color: 'yellow' }
      if (toolName === 'set_status') return { text: `📍 ${agentName} status → ${args.status}${args.note ? `: ${args.note}` : ''}`, color: 'blue' }
      if (toolName === 'create_office_task') return { text: `📋 ${agentName} created task: ${args.title}`, color: 'green' }
      if (toolName === 'update_office_task') return { text: `📋 ${agentName} updated task ${args.id}${args.status ? ` → ${args.status}` : ''}`, color: 'green' }
      if (toolName === 'list_office_tasks') return { text: `📋 ${agentName} checked the Kanban board`, color: 'gray' }
      if (toolName === 'list_statuses') return { text: `📍 ${agentName} checked coworker statuses`, color: 'gray' }
      if (toolName === 'list_coworkers') return { text: `👥 ${agentName} checked the coworker directory`, color: 'gray' }
      if (toolName === 'get_current_time') return { text: `🕒 ${agentName} checked current Unix time`, color: 'blue' }
      if (toolName === 'convert_unix_time_to_iso8601') return { text: `🕒 ${agentName} converted Unix time ${args.unixTime} (${args.unit ?? 'seconds'}) to ISO 8601`, color: 'blue' }
      return { text: `🛠️  ${agentName} called ${toolName}${args ? ` ${JSON.stringify(args)}` : ''}`, color: 'gray' }
    }
    case 'tool-result': {
      const { toolName, result = {} } = chunk.payload ?? {}
      if (toolName === 'ask_human_question') return { text: `❓ ${agentName} queued question ${result.id}`, color: 'yellow' }
      if (toolName === 'set_status') return { text: `📍 ${agentName} status set: ${result.status}${result.note ? ` — ${result.note}` : ''}`, color: 'blue' }
      if (toolName === 'create_office_task') return { text: `📋 Created task ${result.id}: ${result.title}`, color: 'green' }
      if (toolName === 'update_office_task') return { text: `📋 Task ${result.id}: ${result.status} — ${result.title}`, color: 'green' }
      if (toolName === 'list_office_tasks') return { text: `📋 Kanban has ${result.tasks?.length ?? 0} task(s)`, color: 'gray' }
      if (toolName === 'list_statuses') return { text: `📍 Found ${result.statuses?.length ?? 0} coworker status(es)`, color: 'gray' }
      if (toolName === 'list_coworkers') return { text: `👥 Found ${result.coworkers?.length ?? 0} coworker(s)`, color: 'gray' }
      if (toolName === 'list_office_messages') return { text: `✉️  Mailbox has ${result.messages?.length ?? 0} message(s)`, color: 'gray' }
      if (toolName === 'schedule_self_wake') return { text: `⏰ ${agentName} scheduled wake in ${result.delaySeconds}s: ${result.instruction}`, color: 'blue' }
      if (toolName === 'clear_scheduled_action' && result?.cleared) return { text: `🧹 ${agentName} cleared scheduled wake`, color: 'blue' }
      if (toolName === 'get_current_time') return { text: `🕒 ${agentName} current Unix time: ${result.unixTime}`, color: 'blue' }
      if (toolName === 'convert_unix_time_to_iso8601') return { text: `🕒 ${agentName} ISO 8601 time: ${result.iso8601}`, color: 'blue' }
      return { text: `✅ ${agentName} ${toolName} result${result ? ` ${JSON.stringify(result).slice(0, 500)}` : ''}`, color: 'gray' }
    }
    case 'error': {
      const error = chunk.payload?.error
      return { text: `❌ ${agentName} error: ${error?.responseBody ?? error?.message ?? 'unknown error'}`, color: 'red' }
    }
  }
}
