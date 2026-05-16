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
