// Thin browser client for the agent-office API (same-origin).

async function request(method, path, body) {
  const response = await fetch(path, {
    method,
    headers: body === undefined ? undefined : { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  if (!response.ok) {
    let message = `${method} ${path} failed with status ${response.status}`
    try {
      const parsed = await response.json()
      if (parsed.error) message = parsed.error
    } catch {
      // Non-JSON error body; keep the status message.
    }
    throw new Error(message)
  }
  return response.json()
}

export const getState = () => request('GET', '/api/state')
export const sendMessage = (to, body) => request('POST', '/api/messages', { to, body })
export const hire = (name, role) => request('POST', '/api/coworkers', { name, role })
export const fire = name => request('DELETE', `/api/coworkers/${encodeURIComponent(name)}`)
export const rerollAppearance = name => request('POST', `/api/coworkers/${encodeURIComponent(name)}/appearance`)
export const writeWikiPage = (title, content) => request('POST', '/api/wiki', { title, content })
export const deleteWikiPage = slug => request('DELETE', `/api/wiki/${encodeURIComponent(slug)}`)
export const answerQuestion = (id, answer) => request('POST', `/api/questions/${encodeURIComponent(id)}/answer`, answer)
export const saveTeam = path => request('POST', '/api/team/save', { path })
export const loadTeam = path => request('POST', '/api/team/load', { path })

// Native EventSource reconnects automatically and resumes via Last-Event-ID.
export function subscribeEvents(onEvent, onStatus) {
  const source = new EventSource('/api/events')
  source.onopen = () => onStatus?.('connected')
  source.onerror = () => onStatus?.('disconnected')
  source.onmessage = messageEvent => {
    try {
      onEvent(JSON.parse(messageEvent.data))
    } catch {
      // Skip events we cannot parse.
    }
  }
  return () => source.close()
}
