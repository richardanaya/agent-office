import type { CoworkerProfile } from './office/coworkers.js'
import type { CoworkerStatus, OfficeTask } from './office/kanban.js'
import type { HumanQuestion } from './office/human-questions.js'
import type { OfficeMessage } from './office/mailbox.js'
import type { WikiPage } from './office/wiki.js'

// Wire types shared by the office server and its clients (CLI, web).

export type OfficeEvent =
  | { type: 'agent-event'; agent: string; chunk: unknown }
  | { type: 'human-message'; message: OfficeMessage }
  | { type: 'human-message-sent'; to: string; body: string }
  | { type: 'roster-changed'; coworkers: CoworkerProfile[] }

export type StoredOfficeEvent = { id: number; event: OfficeEvent }

export type OfficeState = {
  coworkers: CoworkerProfile[]
  statuses: CoworkerStatus[]
  tasks: OfficeTask[]
  questions: HumanQuestion[]
  wiki: WikiPage[]
  teamFile: string | null
}

export type SendMessageRequest = { to: string; body: string }
export type AnswerQuestionRequest = { selectedChoices?: string[]; customAnswer?: string }
export type HireRequest = { name: string; role: string }
export type TeamPathRequest = { path: string }
export type ApiError = { error: string }
