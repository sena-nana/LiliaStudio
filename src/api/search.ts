import { callCommand } from './client'
import type { SearchFilter, SearchResult } from '@/types/search'

export function searchEntities(filter: SearchFilter): Promise<SearchResult[]> {
  return callCommand<SearchResult[]>('search_entities', { filter }).then((items) => items ?? [])
}
