export function resolveOfficeResourceId() {
  return process.env.AGENT_OFFICE_RESOURCE_ID ?? 'agent-office'
}

export const DEFAULT_OFFICE_PORT = 4747

export function resolveOfficePort() {
  const raw = process.env.AGENT_OFFICE_PORT
  if (!raw) return DEFAULT_OFFICE_PORT
  const port = Number.parseInt(raw, 10)
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new Error(`AGENT_OFFICE_PORT must be a port number, got "${raw}".`)
  }
  return port
}
