import { readFileSync, writeFileSync } from 'node:fs'
import { z } from 'zod'

export const teamFileSchema = z.object({
  name: z.string().trim().min(1).max(120).optional().describe('Optional display name for the team.'),
  coworkers: z
    .array(
      z.object({
        name: z.string().trim().min(2).max(32),
        role: z.string().trim().min(3).max(200),
        appearance: z.number().int().nonnegative().optional().describe('Visual seed for client renderings.'),
      }),
    )
    .max(24),
})

export type TeamFile = z.infer<typeof teamFileSchema>

export function loadTeamFile(path: string): TeamFile {
  let raw: string
  try {
    raw = readFileSync(path, 'utf8')
  } catch (error) {
    throw new Error(`Could not read team file ${path}: ${error instanceof Error ? error.message : String(error)}`)
  }

  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    throw new Error(`Team file ${path} is not valid JSON.`)
  }

  const result = teamFileSchema.safeParse(parsed)
  if (!result.success) {
    const issue = result.error.issues[0]
    throw new Error(`Team file ${path} is invalid: ${issue ? `${issue.path.join('.') || 'team'} — ${issue.message}` : 'unknown error'}`)
  }
  return result.data
}

export function saveTeamFile(path: string, team: TeamFile): void {
  const validated = teamFileSchema.parse(team)
  writeFileSync(path, `${JSON.stringify(validated, null, 2)}\n`, 'utf8')
}
