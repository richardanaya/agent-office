import { Mastra } from '@mastra/core'

// Agents are registered at runtime by hireCoworker in office/hiring.ts.
export const mastra = new Mastra({
  agents: {},
})
