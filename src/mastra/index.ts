import { Mastra } from '@mastra/core'
import { LibSQLStore } from '@mastra/libsql'
import { coworkers, subscribeCoworkerThreads } from '../agents/coworkers.js'
import { resolveStorageUrl } from '../config.js'

subscribeCoworkerThreads()

export const mastra = new Mastra({
  agents: coworkers,
  storage: new LibSQLStore({
    id: 'office-storage',
    url: resolveStorageUrl(),
  }),
})
