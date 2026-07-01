export type CoworkerProfile = {
  name: string
  role: string
}

// The office starts empty; coworkers are hired at runtime (interactively or
// from a team file) via hireCoworker in hiring.ts.
export const agentCoworkerProfiles: CoworkerProfile[] = []

export const humanProfile: CoworkerProfile = {
  name: 'Human',
  role: 'the human teammate who can assign work, answer questions, and receive updates',
}

export function listAgentCoworkers(): CoworkerProfile[] {
  return [...agentCoworkerProfiles]
}

export function addAgentCoworker(profile: CoworkerProfile): CoworkerProfile {
  const name = profile.name.trim()
  const role = profile.role.trim()
  if (!/^[A-Za-z][A-Za-z0-9_-]{1,31}$/.test(name)) {
    throw new Error('Coworker name must start with a letter and contain 2-32 letters, numbers, underscores, or hyphens.')
  }
  if (role.length < 3 || role.length > 200) {
    throw new Error('Coworker role must be 3-200 characters.')
  }
  if (agentCoworkerProfiles.some(coworker => coworker.name.toLowerCase() === name.toLowerCase())) {
    throw new Error(`Coworker ${name} already exists.`)
  }
  const coworker = { name, role }
  agentCoworkerProfiles.push(coworker)
  return coworker
}

export function removeAgentCoworker(name: string): CoworkerProfile | undefined {
  const index = agentCoworkerProfiles.findIndex(coworker => coworker.name.toLowerCase() === name.trim().toLowerCase())
  if (index < 0) return undefined
  return agentCoworkerProfiles.splice(index, 1)[0]
}

export function listCoworkerProfiles(): CoworkerProfile[] {
  return [...agentCoworkerProfiles, humanProfile]
}
