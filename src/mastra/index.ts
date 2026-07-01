import { Mastra } from '@mastra/core'
import { coworkers, subscribeCoworkerThreads } from '../agents/coworkers.js'

subscribeCoworkerThreads()

export const mastra = new Mastra({
  agents: coworkers,
})
