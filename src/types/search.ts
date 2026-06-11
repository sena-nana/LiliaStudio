export interface SearchFilter {
  projectId: string
  query: string
  entityTypes: string[]
}

export interface SearchResult {
  entityType: string
  entityId: string
  title: string
  snippet: string
  score: number
}

export interface SemanticSearchRequest {
  projectId: string
  query: string
  limit: number
}

export type SemanticSearchStatus = 'idle' | 'ready' | 'degraded' | 'failed'

export type SemanticSearchItem = SearchResult

export interface SemanticSearchResponse {
  status: Exclude<SemanticSearchStatus, 'idle'>
  message: string
  model: string
  items: SemanticSearchItem[]
}

export interface MixedSearchResult extends SearchResult {
  source: 'keyword' | 'semantic'
}
