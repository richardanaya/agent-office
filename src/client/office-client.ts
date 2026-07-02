import type { CoworkerProfile } from '../office/coworkers.js'
import type { HumanQuestion } from '../office/human-questions.js'
import type { OfficeMessage } from '../office/mailbox.js'
import type { AnswerQuestionRequest, OfficeState, StoredOfficeEvent } from '../protocol.js'

const RECONNECT_DELAY_MS = 2_000

export type EventStreamStatus = 'connected' | 'disconnected'

export type SubscribeOptions = {
  onStatus?: (status: EventStreamStatus) => void
}

function parseSseMessage(raw: string): { id?: number; data?: string } {
  let id: number | undefined
  const dataLines: string[] = []
  for (const line of raw.split('\n')) {
    if (line.startsWith('id:')) id = Number(line.slice(3).trim())
    else if (line.startsWith('data:')) dataLines.push(line.slice(5).trimStart())
  }
  return { id: Number.isFinite(id) ? id : undefined, data: dataLines.length > 0 ? dataLines.join('\n') : undefined }
}

export class OfficeClient {
  readonly baseUrl: string

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/+$/, '')
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: body === undefined ? undefined : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    })
    if (!response.ok) {
      let message = `${method} ${path} failed with status ${response.status}`
      try {
        const parsed = (await response.json()) as { error?: string }
        if (parsed.error) message = parsed.error
      } catch {
        // Non-JSON error body; keep the status message.
      }
      throw new Error(message)
    }
    return (await response.json()) as T
  }

  getState(): Promise<OfficeState> {
    return this.request('GET', '/api/state')
  }

  sendMessage(to: string, body: string): Promise<{ message: OfficeMessage }> {
    return this.request('POST', '/api/messages', { to, body })
  }

  answerQuestion(id: string, answer: AnswerQuestionRequest): Promise<{ question: HumanQuestion }> {
    return this.request('POST', `/api/questions/${encodeURIComponent(id)}/answer`, answer)
  }

  hire(name: string, role: string): Promise<{ profile: CoworkerProfile }> {
    return this.request('POST', '/api/coworkers', { name, role })
  }

  fire(name: string): Promise<{ profile: CoworkerProfile }> {
    return this.request('DELETE', `/api/coworkers/${encodeURIComponent(name)}`)
  }

  saveTeam(path: string): Promise<{ path: string; coworkers: CoworkerProfile[] }> {
    return this.request('POST', '/api/team/save', { path })
  }

  loadTeam(path: string): Promise<{ path: string; name?: string; coworkers: CoworkerProfile[]; skipped: string[] }> {
    return this.request('POST', '/api/team/load', { path })
  }

  // Long-lived SSE subscription with automatic reconnect; replays missed
  // events via ?since=<last seen id>. Returns a stop function.
  subscribeEvents(onEvent: (event: StoredOfficeEvent) => void, options: SubscribeOptions = {}): () => void {
    let stopped = false
    let lastId = 0
    const controller = new AbortController()

    const run = async () => {
      while (!stopped) {
        try {
          const response = await fetch(`${this.baseUrl}/api/events?since=${lastId}`, {
            headers: { accept: 'text/event-stream' },
            signal: controller.signal,
          })
          if (!response.ok || !response.body) throw new Error(`event stream failed with status ${response.status}`)
          options.onStatus?.('connected')
          const reader = response.body.getReader()
          const decoder = new TextDecoder()
          let buffer = ''
          while (true) {
            const { done, value } = await reader.read()
            if (done) break
            buffer += decoder.decode(value, { stream: true })
            let separator = buffer.indexOf('\n\n')
            while (separator !== -1) {
              const raw = buffer.slice(0, separator)
              buffer = buffer.slice(separator + 2)
              separator = buffer.indexOf('\n\n')
              const { id, data } = parseSseMessage(raw)
              if (id !== undefined) lastId = id
              if (data === undefined) continue
              try {
                onEvent({ id: id ?? 0, event: JSON.parse(data) })
              } catch {
                // Skip events we cannot parse rather than killing the stream.
              }
            }
          }
          throw new Error('event stream ended')
        } catch {
          if (stopped) return
          options.onStatus?.('disconnected')
          await new Promise(resolve => setTimeout(resolve, RECONNECT_DELAY_MS))
        }
      }
    }

    void run()
    return () => {
      stopped = true
      controller.abort()
    }
  }
}
