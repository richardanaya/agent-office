import { resolveOfficeResourceId } from '../config.js'
import { normalizeCoworkerName } from './names.js'

export const OFFICE_RESOURCE_ID = resolveOfficeResourceId()

export function coworkerThread(coworkerName: string) {
  return {
    resourceId: OFFICE_RESOURCE_ID,
    threadId: `coworker:${normalizeCoworkerName(coworkerName)}`,
  }
}
