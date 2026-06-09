import { executeCommand, listResult } from './client'
import { commands } from './commands'
import type { SearchFilter, SearchResult } from '@/types/search'

export function searchEntities(filter: SearchFilter): Promise<SearchResult[]> {
  return executeCommand(commands.search.entities, { filter }).then(listResult)
}
