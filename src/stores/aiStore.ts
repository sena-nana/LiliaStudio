import { defineStore } from 'pinia'
import { createAiJob, defaultAiProviders, indexChunks, listAiJobs, listPromptTemplates, previewChunks } from '@/api/ai'
import type { AiJob, AiJobDraft, AiProviderConfig, DocumentChunkRecord, PromptTemplate, TextChunk } from '@/types/ai'

interface AiState {
  providers: AiProviderConfig[]
  chunks: DocumentChunkRecord[]
  preview: TextChunk[]
  jobs: AiJob[]
  prompts: PromptTemplate[]
  loading: boolean
}

export const useAiStore = defineStore('ai', {
  state: (): AiState => ({
    providers: [],
    chunks: [],
    preview: [],
    jobs: [],
    prompts: [],
    loading: false,
  }),
  actions: {
    async loadDefaults() {
      this.providers = await defaultAiProviders()
    },
    async loadPromptsAndJobs() {
      const [prompts, jobs] = await Promise.all([listPromptTemplates(), listAiJobs()])
      this.prompts = prompts
      this.jobs = jobs
    },
    async createJob(draft: AiJobDraft) {
      const job = await createAiJob(draft)
      this.jobs = [job, ...this.jobs]
      return job
    },
    async previewText(text: string, maxChars = 600) {
      this.preview = await previewChunks(text, maxChars)
    },
    async indexProject(projectId: string, maxChars = 600) {
      this.loading = true
      try {
        this.chunks = await indexChunks(projectId, maxChars)
      } finally {
        this.loading = false
      }
    },
  },
})
