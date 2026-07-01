import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { loadTeamFile, saveTeamFile } from '../src/office/team.js'

function tempPath(name: string) {
  return join(mkdtempSync(join(tmpdir(), 'agent-office-test-')), name)
}

describe('team files', () => {
  it('round-trips a team through save and load', () => {
    const path = tempPath('team.json')
    const team = { name: 'Dream Team', coworkers: [{ name: 'Dana', role: 'data analyst' }, { name: 'Ed', role: 'editor' }] }
    saveTeamFile(path, team)
    expect(loadTeamFile(path)).toEqual(team)
  })

  it('writes human-editable pretty JSON', () => {
    const path = tempPath('team.json')
    saveTeamFile(path, { coworkers: [{ name: 'Dana', role: 'data analyst' }] })
    expect(readFileSync(path, 'utf8')).toContain('\n  "coworkers"')
  })

  it('throws a readable error for a missing file', () => {
    expect(() => loadTeamFile(tempPath('missing.json'))).toThrow(/Could not read team file/)
  })

  it('throws a readable error for invalid JSON', () => {
    const path = tempPath('broken.json')
    writeFileSync(path, '{ not json')
    expect(() => loadTeamFile(path)).toThrow(/not valid JSON/)
  })

  it('rejects files that do not match the team shape', () => {
    const path = tempPath('wrong.json')
    writeFileSync(path, JSON.stringify({ coworkers: [{ name: 'x' }] }))
    expect(() => loadTeamFile(path)).toThrow(/invalid/)
  })

  it('validates before saving', () => {
    const path = tempPath('never-written.json')
    expect(() => saveTeamFile(path, { coworkers: [{ name: 'D', role: 'ok role' }] })).toThrow()
  })
})
