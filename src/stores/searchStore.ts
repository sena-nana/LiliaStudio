import { defineStore } from 'pinia'
import { searchEntities } from '@/api/search'
import type { SearchResult } from '@/types/search'

interface SearchState {
  query: string
  results: SearchResult[]
  loading: boolean
}

export const useSearchStore = defineStore('search', {
  state: (): SearchState => ({
    query: '',
    results: [],
    loading: false,
  }),
  actions: {
    async run(projectId: string, query: string) {
      this.query = query
      this.loading = true
      try {
        this.results = await searchEntities({ projectId, query, entityTypes: [] })
      } finally {
        this.loading = false
      }
    },
  },
})
