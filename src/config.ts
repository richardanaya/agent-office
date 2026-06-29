export function resolveStorageUrl() {
  return process.env.AGENT_OFFICE_DB_URL ?? 'file:./agent-office.db'
}

export function resolveOfficeResourceId() {
  return process.env.AGENT_OFFICE_RESOURCE_ID ?? 'agent-office'
}