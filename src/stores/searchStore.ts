import { defineStore } from 'pinia'
import { searchEntities, searchSemantic } from '@/api/search'
import type {
  MixedSearchResult,
  SearchResult,
  SemanticSearchItem,
  SemanticSearchStatus,
} from '@/types/search'

interface SearchState {
  query: string
  keywordResults: SearchResult[]
  semanticResults: SemanticSearchItem[]
  loading: boolean
  semanticStatus: SemanticSearchStatus
  semanticMessage: string
}

export const useSearchStore = defineStore('search', {
  state: (): SearchState => ({
    query: '',
    keywordResults: [],
    semanticResults: [],
    loading: false,
    semanticStatus: 'idle',
    semanticMessage: '',
  }),
  getters: {
    results(state): MixedSearchResult[] {
      const seen = new Set<string>()
      const keywordResults = state.keywordResults.map((result) => {
        seen.add(resultKey(result))
        return { ...result, source: 'keyword' as const }
      })
      const semanticResults = state.semanticResults
        .filter((result) => !seen.has(resultKey(result)))
        .map((result) => ({ ...result, source: 'semantic' as const }))

      return [...keywordResults, ...semanticResults]
    },
  },
  actions: {
    async run(projectId: string, query: string) {
      this.query = query
      this.loading = true
      this.semanticStatus = 'idle'
      this.semanticMessage = ''
      try {
        const [keywordResults, semanticResponse] = await Promise.all([
          searchEntities({ projectId, query, entityTypes: [] }),
          searchSemantic({ projectId, query, limit: 8 }).catch((error: unknown) => ({
            status: 'failed' as const,
            message: error instanceof Error ? error.message : String(error),
            model: '',
            items: [],
          })),
        ])
        this.keywordResults = keywordResults
        this.semanticResults = semanticResponse.items
        this.semanticStatus = semanticResponse.status
        this.semanticMessage = semanticResponse.message
      } finally {
        this.loading = false
      }
    },
  },
})

function resultKey(result: Pick<SearchResult, 'entityType' | 'entityId'>): string {
  return `${result.entityType}:${result.entityId}`
}
