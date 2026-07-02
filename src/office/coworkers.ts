export type CoworkerProfile = {
  name: string
  role: string
  // Visual seed for client renderings (e.g. the web villager); rerollable.
  appearance?: number
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
  if (profile.appearance !== undefined && (!Number.isInteger(profile.appearance) || profile.appearance < 0)) {
    throw new Error('Coworker appearance must be a non-negative integer.')
  }
  if (agentCoworkerProfiles.some(coworker => coworker.name.toLowerCase() === name.toLowerCase())) {
    throw new Error(`Coworker ${name} already exists.`)
  }
  const coworker: CoworkerProfile = { name, role, ...(profile.appearance !== undefined ? { appearance: profile.appearance } : {}) }
  agentCoworkerProfiles.push(coworker)
  return { ...coworker }
}

export function setCoworkerAppearance(name: string, appearance: number): CoworkerProfile {
  if (!Number.isInteger(appearance) || appearance < 0) {
    throw new Error('Coworker appearance must be a non-negative integer.')
  }
  const profile = agentCoworkerProfiles.find(coworker => coworker.name.toLowerCase() === name.trim().toLowerCase())
  if (!profile) throw new Error(`No coworker named ${name}.`)
  profile.appearance = appearance
  return { ...profile }
}

export function removeAgentCoworker(name: string): CoworkerProfile | undefined {
  const index = agentCoworkerProfiles.findIndex(coworker => coworker.name.toLowerCase() === name.trim().toLowerCase())
  if (index < 0) return undefined
  return agentCoworkerProfiles.splice(index, 1)[0]
}

export function listCoworkerProfiles(): CoworkerProfile[] {
  return [...agentCoworkerProfiles, humanProfile]
}
