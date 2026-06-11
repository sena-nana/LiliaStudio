import { executeCommand, listResult } from './client'
import { commands } from './commands'
import type {
  SearchFilter,
  SearchResult,
  SemanticSearchRequest,
  SemanticSearchResponse,
} from '@/types/search'

export function searchEntities(filter: SearchFilter): Promise<SearchResult[]> {
  return executeCommand(commands.search.entities, { filter }).then(listResult)
}

export function searchSemantic(request: SemanticSearchRequest): Promise<SemanticSearchResponse> {
  return executeCommand(commands.search.semantic, { request })
}
