import { Mastra } from '@mastra/core'
import { coworkers } from '../agents/coworkers.js'

export const mastra = new Mastra({
  agents: coworkers,
})
