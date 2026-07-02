import { readFile } from 'node:fs/promises'
import { createServer, type IncomingMessage, type ServerResponse } from 'node:http'
import { createRequire } from 'node:module'
import type { AddressInfo } from 'node:net'
import { dirname, extname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import type { Agent } from '@mastra/core/agent'
import { getCoworkerAgent } from '../agents/coworkers.js'
import { listAgentCoworkers } from '../office/coworkers.js'
import { fireCoworker, hireCoworker } from '../office/hiring.js'
import { deliverHumanMessage, listHumanInbox, markAllHumanMessagesSeen } from '../office/human.js'
import { answerHumanQuestion, listHumanQuestions } from '../office/human-questions.js'
import { listCoworkerStatuses, listOfficeTasks } from '../office/kanban.js'
import { loadTeamFile, saveTeamFile, type TeamFile } from '../office/team.js'
import { coworkerThread } from '../office/threads.js'
import type { OfficeState, StoredOfficeEvent } from '../protocol.js'
import { OfficeEventBus } from './event-bus.js'

const INBOX_PUMP_INTERVAL_MS = 500
const SSE_HEARTBEAT_MS = 30_000
const MAX_BODY_BYTES = 1_048_576

// The static web UI ships in <package root>/web; this file compiles to
// dist/server/, so ../../web resolves correctly from both src and dist.
const WEB_ROOT = fileURLToPath(new URL('../../web', import.meta.url))

const MIME_TYPES: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
}

// three.js is served from the installed package so the web UI needs no CDN
// or build step. Resolved lazily so a missing install degrades to a 404.
function resolveThreeRoots(): { build: string; addons: string } | undefined {
  try {
    // three does not export ./package.json; its main entry lives in build/.
    const require = createRequire(import.meta.url)
    const build = dirname(require.resolve('three'))
    return { build, addons: join(dirname(build), 'examples', 'jsm') }
  } catch {
    return undefined
  }
}

const threeRoots = resolveThreeRoots()

async function serveStaticFile(res: ServerResponse, root: string, relativePath: string): Promise<void> {
  const filePath = resolve(root, relativePath.replace(/^\/+/, ''))
  if (filePath !== root && !filePath.startsWith(root + sep)) {
    return sendJson(res, 404, { error: 'Not found.' })
  }
  try {
    const contents = await readFile(filePath)
    res.writeHead(200, { 'content-type': MIME_TYPES[extname(filePath)] ?? 'application/octet-stream' })
    res.end(contents)
  } catch {
    sendJson(res, 404, { error: 'Not found.' })
  }
}

export type OfficeServerOptions = {
  port?: number
  host?: string
  teamFile?: string
}

export type OfficeServer = {
  url: string
  port: number
  bus: OfficeEventBus
  // Team members that failed to hire during startup, as readable reasons.
  bootstrapWarnings: string[]
  close(): Promise<void>
}

function sendJson(res: ServerResponse, status: number, body: unknown) {
  res.writeHead(status, { 'content-type': 'application/json' })
  res.end(JSON.stringify(body))
}

async function readJson(req: IncomingMessage): Promise<Record<string, unknown>> {
  const chunks: Buffer[] = []
  let size = 0
  for await (const chunk of req) {
    size += (chunk as Buffer).length
    if (size > MAX_BODY_BYTES) throw new Error('Request body too large.')
    chunks.push(chunk as Buffer)
  }
  const raw = Buffer.concat(chunks).toString('utf8')
  if (!raw) return {}
  const parsed: unknown = JSON.parse(raw)
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) throw new Error('Request body must be a JSON object.')
  return parsed as Record<string, unknown>
}

function requireString(body: Record<string, unknown>, field: string): string {
  const value = body[field]
  if (typeof value !== 'string' || value.trim() === '') throw new Error(`"${field}" must be a non-empty string.`)
  return value
}

export async function startOfficeServer(options: OfficeServerOptions = {}): Promise<OfficeServer> {
  const bus = new OfficeEventBus()
  const agentWatchers = new Map<string, { unsubscribe: () => void }>()
  const sseClients = new Set<ServerResponse>()
  const bootstrapWarnings: string[] = []
  let teamFilePath: string | null = null
  let closed = false

  async function watchAgent(name: string, agent: Agent) {
    const key = name.toLowerCase()
    if (agentWatchers.has(key)) return
    try {
      const subscription = await agent.subscribeToThread(coworkerThread(name))
      agentWatchers.set(key, subscription)
      for await (const chunk of subscription.stream) {
        bus.publish({ type: 'agent-event', agent: name, chunk })
      }
    } catch (error) {
      bus.publish({
        type: 'agent-event',
        agent: name,
        chunk: { type: 'error', payload: { error: { message: `stopped watching: ${error instanceof Error ? error.message : String(error)}` } } },
      })
    } finally {
      agentWatchers.delete(key)
    }
  }

  function unwatchAgent(name: string) {
    const key = name.toLowerCase()
    agentWatchers.get(key)?.unsubscribe()
    agentWatchers.delete(key)
  }

  function publishRoster() {
    bus.publish({ type: 'roster-changed', coworkers: listAgentCoworkers() })
  }

  function hireAndWatch(profile: { name: string; role: string }, announce: boolean) {
    const hired = hireCoworker(profile, { announce })
    void watchAgent(hired.profile.name, hired.agent)
    return hired
  }

  function fireAndUnwatch(name: string) {
    const removed = fireCoworker(name)
    if (removed) unwatchAgent(removed.name)
    return removed
  }

  function applyTeam(team: TeamFile): string[] {
    for (const existing of listAgentCoworkers()) fireAndUnwatch(existing.name)
    const failures: string[] = []
    for (const member of team.coworkers) {
      try {
        hireAndWatch(member, false)
      } catch (error) {
        failures.push(`${member.name} (${error instanceof Error ? error.message : String(error)})`)
      }
    }
    return failures
  }

  function currentState(): OfficeState {
    return {
      coworkers: listAgentCoworkers(),
      statuses: listCoworkerStatuses(),
      tasks: listOfficeTasks(),
      questions: listHumanQuestions({ unansweredOnly: true }),
      teamFile: teamFilePath,
    }
  }

  // Bootstrap: load the requested team, and watch any coworkers hired before
  // the server started (library users may hire first, then start the server).
  if (options.teamFile) {
    const team = loadTeamFile(options.teamFile)
    bootstrapWarnings.push(...applyTeam(team).map(failure => `Skipped ${failure}`))
    teamFilePath = options.teamFile
  }
  for (const profile of listAgentCoworkers()) {
    const agent = getCoworkerAgent(profile.name)
    if (agent) void watchAgent(profile.name, agent)
  }

  // The server owns Human inbox delivery: broadcast to every connected client
  // and mark seen once, so multiple clients never race over unread messages.
  const inboxPump = setInterval(() => {
    const unread = listHumanInbox({ unreadOnly: true })
    if (unread.length === 0) return
    for (const message of unread) bus.publish({ type: 'human-message', message })
    markAllHumanMessagesSeen()
  }, INBOX_PUMP_INTERVAL_MS)

  function writeSseEvent(res: ServerResponse, stored: StoredOfficeEvent) {
    try {
      res.write(`id: ${stored.id}\ndata: ${JSON.stringify(stored.event)}\n\n`)
    } catch {
      // Unserializable chunk or closed socket; drop this event for this client.
    }
  }

  function handleEventStream(req: IncomingMessage, res: ServerResponse, url: URL) {
    res.writeHead(200, {
      'content-type': 'text/event-stream',
      'cache-control': 'no-cache',
      connection: 'keep-alive',
      'access-control-allow-origin': '*',
    })
    res.write(':connected\n\n')
    sseClients.add(res)
    // Browsers' native EventSource resumes with a Last-Event-ID header;
    // other clients pass ?since=<id>.
    const lastEventIdHeader = Number(req.headers['last-event-id'])
    const since = Number(url.searchParams.get('since') ?? (Number.isFinite(lastEventIdHeader) ? lastEventIdHeader : 0))
    for (const stored of bus.eventsSince(Number.isFinite(since) ? since : 0)) writeSseEvent(res, stored)
    const unsubscribe = bus.subscribe(stored => writeSseEvent(res, stored))
    const heartbeat = setInterval(() => {
      try {
        res.write(':heartbeat\n\n')
      } catch {
        // Socket already gone; close cleanup below handles it.
      }
    }, SSE_HEARTBEAT_MS)
    req.on('close', () => {
      unsubscribe()
      clearInterval(heartbeat)
      sseClients.delete(res)
    })
  }

  async function route(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const url = new URL(req.url ?? '/', 'http://localhost')
    const path = url.pathname
    const method = req.method ?? 'GET'

    if (method === 'GET' && path === '/api/state') return sendJson(res, 200, currentState())
    if (method === 'GET' && path === '/api/events') return handleEventStream(req, res, url)

    if (method === 'POST' && path === '/api/messages') {
      const body = await readJson(req)
      const to = requireString(body, 'to')
      const message = requireString(body, 'body')
      const sent = await deliverHumanMessage(to, message)
      bus.publish({ type: 'human-message-sent', to, body: message })
      return sendJson(res, 200, { message: sent })
    }

    if (method === 'POST' && path === '/api/coworkers') {
      const body = await readJson(req)
      const hired = hireAndWatch({ name: requireString(body, 'name'), role: requireString(body, 'role') }, true)
      publishRoster()
      return sendJson(res, 201, { profile: hired.profile })
    }

    const coworkerMatch = /^\/api\/coworkers\/([^/]+)$/.exec(path)
    if (method === 'DELETE' && coworkerMatch) {
      const removed = fireAndUnwatch(decodeURIComponent(coworkerMatch[1]!))
      if (!removed) return sendJson(res, 404, { error: `No coworker named ${decodeURIComponent(coworkerMatch[1]!)}.` })
      publishRoster()
      return sendJson(res, 200, { profile: removed })
    }

    const answerMatch = /^\/api\/questions\/([^/]+)\/answer$/.exec(path)
    if (method === 'POST' && answerMatch) {
      const body = await readJson(req)
      const selectedChoices = Array.isArray(body.selectedChoices) ? body.selectedChoices.filter((choice): choice is string => typeof choice === 'string') : []
      const customAnswer = typeof body.customAnswer === 'string' ? body.customAnswer : undefined
      const question = answerHumanQuestion({ id: decodeURIComponent(answerMatch[1]!), selectedChoices, customAnswer })
      return sendJson(res, 200, { question })
    }

    if (method === 'POST' && path === '/api/team/save') {
      const body = await readJson(req)
      const targetPath = requireString(body, 'path')
      saveTeamFile(targetPath, { coworkers: listAgentCoworkers() })
      teamFilePath = targetPath
      return sendJson(res, 200, { path: targetPath, coworkers: listAgentCoworkers() })
    }

    if (method === 'POST' && path === '/api/team/load') {
      const body = await readJson(req)
      const sourcePath = requireString(body, 'path')
      const team = loadTeamFile(sourcePath)
      const failures = applyTeam(team)
      teamFilePath = sourcePath
      publishRoster()
      return sendJson(res, 200, { path: sourcePath, name: team.name, coworkers: listAgentCoworkers(), skipped: failures })
    }

    // Everything outside /api is the static web UI.
    if (method === 'GET' && !path.startsWith('/api/')) {
      if (path.startsWith('/vendor/')) {
        if (!threeRoots) return sendJson(res, 404, { error: 'three.js is not installed on the server.' })
        if (path.startsWith('/vendor/addons/')) return serveStaticFile(res, threeRoots.addons, decodeURIComponent(path.slice('/vendor/addons/'.length)))
        return serveStaticFile(res, threeRoots.build, decodeURIComponent(path.slice('/vendor/'.length)))
      }
      return serveStaticFile(res, WEB_ROOT, decodeURIComponent(path === '/' ? 'index.html' : path))
    }

    sendJson(res, 404, { error: `No route for ${method} ${path}.` })
  }

  const server = createServer((req, res) => {
    res.setHeader('access-control-allow-origin', '*')
    res.setHeader('access-control-allow-methods', 'GET, POST, DELETE, OPTIONS')
    res.setHeader('access-control-allow-headers', 'content-type')
    if (req.method === 'OPTIONS') {
      res.writeHead(204)
      res.end()
      return
    }
    route(req, res).catch(error => {
      const message = error instanceof Error ? error.message : String(error)
      if (!res.headersSent) sendJson(res, 400, { error: message })
      else res.end()
    })
  })

  await new Promise<void>((resolve, reject) => {
    server.once('error', reject)
    server.listen(options.port ?? 0, options.host ?? '127.0.0.1', () => {
      server.removeListener('error', reject)
      resolve()
    })
  })

  const address = server.address() as AddressInfo
  const host = options.host ?? '127.0.0.1'
  const url = `http://${host === '0.0.0.0' || host === '::' ? '127.0.0.1' : host}:${address.port}`

  return {
    url,
    port: address.port,
    bus,
    bootstrapWarnings,
    close: async () => {
      if (closed) return
      closed = true
      clearInterval(inboxPump)
      for (const res of sseClients) res.end()
      sseClients.clear()
      for (const name of [...agentWatchers.keys()]) unwatchAgent(name)
      await new Promise<void>((resolve, reject) => {
        server.close(error => (error ? reject(error) : resolve()))
      })
    },
  }
}
