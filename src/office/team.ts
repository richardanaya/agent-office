import { readFileSync, writeFileSync } from 'node:fs'
import { z } from 'zod'

const taskFileSchema = z.object({
  id: z.string().min(1),
  title: z.string().trim().min(1).max(120),
  description: z.string().max(1000).optional(),
  status: z.enum(['backlog', 'todo', 'doing', 'blocked', 'review', 'done', 'canceled']),
  assignee: z.string().max(64).optional(),
  priority: z.enum(['low', 'normal', 'high', 'urgent']),
  createdBy: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
})

const wikiPageFileSchema = z.object({
  slug: z.string().min(1).max(160),
  title: z.string().trim().min(2).max(120),
  content: z.string().min(1).max(20_000),
  updatedBy: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
})

// A team file is a whole saved office: roster, kanban board, and wiki.
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
  tasks: z.array(taskFileSchema).max(500).optional().describe('Kanban board tasks.'),
  wiki: z.array(wikiPageFileSchema).max(200).optional().describe('Office wiki pages.'),
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
