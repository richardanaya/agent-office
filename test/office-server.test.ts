import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest'
import { OfficeClient } from '../src/client/office-client.js'
import { startOfficeServer, type OfficeServer } from '../src/server/office-server.js'
import { askHumanQuestion } from '../src/office/human-questions.js'
import { officeMailbox } from '../src/office/mailbox.js'
import type { StoredOfficeEvent } from '../src/protocol.js'

let server: OfficeServer
let client: OfficeClient

beforeAll(async () => {
  server = await startOfficeServer({ port: 0 })
  client = new OfficeClient(server.url)
})

afterAll(async () => {
  await server.close()
})

describe('office server API', () => {
  it('serves the office state', async () => {
    const state = await client.getState()
    expect(state.coworkers).toEqual([])
    expect(state.tasks).toEqual([])
    expect(state.questions).toEqual([])
    expect(state.teamFile).toBeNull()
  })

  it('hires and fires coworkers', async () => {
    const { profile } = await client.hire('Remo', 'remote work specialist')
    expect(profile.name).toBe('Remo')
    expect((await client.getState()).coworkers.map(coworker => coworker.name)).toContain('Remo')

    const removed = await client.fire('remo')
    expect(removed.profile.name).toBe('Remo')
    expect((await client.getState()).coworkers).toEqual([])
  })

  it('returns readable errors for invalid requests', async () => {
    await expect(client.hire('1bad', 'some role')).rejects.toThrow(/must start with a letter/)
    await expect(client.fire('nobody')).rejects.toThrow(/No coworker named nobody/)
    await expect(client.sendMessage('ghost', 'hello')).rejects.toThrow(/No coworker agent found/)
  })

  it('answers questions through the API', async () => {
    const question = askHumanQuestion({ from: 'Remo', prompt: 'Ship it?', choices: ['yes', 'no'], mode: 'single' })
    const { question: answered } = await client.answerQuestion(question.id, { selectedChoices: ['yes'] })
    expect(answered.answer?.selectedChoices).toEqual(['yes'])
    await expect(client.answerQuestion(question.id, { selectedChoices: ['no'] })).rejects.toThrow(/already been answered/)
  })

  it('saves and loads teams through the API', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'agent-office-server-'))
    await client.hire('Sava', 'savings specialist')
    const saved = await client.saveTeam(join(dir, 'crew.json'))
    expect(saved.coworkers.map(coworker => coworker.name)).toEqual(['Sava'])

    const otherTeam = join(dir, 'other.json')
    writeFileSync(otherTeam, JSON.stringify({ coworkers: [{ name: 'Lodi', role: 'loader of teams' }] }))
    const loaded = await client.loadTeam(otherTeam)
    expect(loaded.coworkers.map(coworker => coworker.name)).toEqual(['Lodi'])
    expect(loaded.skipped).toEqual([])

    const state = await client.getState()
    expect(state.coworkers.map(coworker => coworker.name)).toEqual(['Lodi'])
    expect(state.teamFile).toBe(otherTeam)
    await client.fire('Lodi')
  })

  it('rerolls a coworker appearance and persists it in team files', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'agent-office-appearance-'))
    await client.hire('Dicey', 'appearance tester')
    try {
      const { profile } = await client.rerollAppearance('Dicey')
      expect(Number.isInteger(profile.appearance)).toBe(true)
      const state = await client.getState()
      expect(state.coworkers.find(coworker => coworker.name === 'Dicey')?.appearance).toBe(profile.appearance)

      const saved = await client.saveTeam(join(dir, 'dicey.json'))
      expect(saved.coworkers[0]?.appearance).toBe(profile.appearance)
    } finally {
      await client.fire('Dicey')
    }
  })

  it('rejects loading a team file that does not exist', async () => {
    await expect(client.loadTeam('/definitely/missing.json')).rejects.toThrow(/Could not read team file/)
  })

  it('streams human inbox messages over SSE and marks them seen once', async () => {
    const received: StoredOfficeEvent[] = []
    const stop = client.subscribeEvents(event => received.push(event))
    try {
      officeMailbox.send({ from: 'zed', to: 'Human', body: 'ping from zed' })
      await vi.waitFor(() => {
        expect(received.some(({ event }) => event.type === 'human-message' && event.message.body === 'ping from zed')).toBe(true)
      }, { timeout: 3_000 })
      expect(officeMailbox.undeliveredFor('Human')).toEqual([])
    } finally {
      stop()
    }
  })

  it('replays buffered events to late subscribers via ?since', async () => {
    officeMailbox.send({ from: 'zed', to: 'Human', body: 'buffered ping' })
    await vi.waitFor(() => {
      expect(officeMailbox.undeliveredFor('Human')).toEqual([])
    }, { timeout: 3_000 })

    const received: StoredOfficeEvent[] = []
    const stop = client.subscribeEvents(event => received.push(event))
    try {
      await vi.waitFor(() => {
        expect(received.some(({ event }) => event.type === 'human-message' && event.message.body === 'buffered ping')).toBe(true)
      }, { timeout: 3_000 })
    } finally {
      stop()
    }
  })

  it('broadcasts roster changes as events', async () => {
    const received: StoredOfficeEvent[] = []
    const stop = client.subscribeEvents(event => received.push(event))
    try {
      await client.hire('Rost', 'roster tester')
      await vi.waitFor(() => {
        expect(received.some(({ event }) => event.type === 'roster-changed' && event.coworkers.some(coworker => coworker.name === 'Rost'))).toBe(true)
      }, { timeout: 3_000 })
    } finally {
      stop()
      await client.fire('Rost')
    }
  })

  it('404s unknown routes', async () => {
    const response = await fetch(`${server.url}/api/nope`)
    expect(response.status).toBe(404)
  })
})

describe('static web UI', () => {
  it('serves the web app at the root', async () => {
    const response = await fetch(`${server.url}/`)
    expect(response.status).toBe(200)
    expect(response.headers.get('content-type')).toContain('text/html')
    expect(await response.text()).toContain('Agent Office')
  })

  it('serves the app modules and styles', async () => {
    for (const path of ['/js/app.js', '/js/scene.js', '/js/api.js']) {
      const response = await fetch(`${server.url}${path}`)
      expect(response.status).toBe(200)
      expect(response.headers.get('content-type')).toContain('text/javascript')
    }
    const css = await fetch(`${server.url}/style.css`)
    expect(css.headers.get('content-type')).toContain('text/css')
  })

  it('serves three.js from the installed package', async () => {
    const module = await fetch(`${server.url}/vendor/three.module.js`)
    expect(module.status).toBe(200)
    const addon = await fetch(`${server.url}/vendor/addons/controls/OrbitControls.js`)
    expect(addon.status).toBe(200)
  })

  it('blocks path traversal out of the static roots', async () => {
    for (const path of ['/%2e%2e/package.json', '/vendor/%2e%2e/package.json', '/vendor/addons/%2e%2e/%2e%2e/package.json']) {
      const response = await fetch(`${server.url}${path}`)
      expect(response.status).toBe(404)
    }
  })

  it('resumes the event stream from the Last-Event-ID header', async () => {
    officeMailbox.send({ from: 'zed', to: 'Human', body: 'resume ping' })
    await vi.waitFor(() => {
      expect(officeMailbox.undeliveredFor('Human')).toEqual([])
    }, { timeout: 3_000 })

    const controller = new AbortController()
    const response = await fetch(`${server.url}/api/events`, {
      headers: { accept: 'text/event-stream', 'last-event-id': '0' },
      signal: controller.signal,
    })
    const reader = response.body!.getReader()
    const { value } = await reader.read()
    controller.abort()
    expect(new TextDecoder().decode(value)).toContain('resume ping')
  })
})
