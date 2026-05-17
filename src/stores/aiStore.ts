import { defineStore } from 'pinia'
import {
  createAiJob,
  defaultAiProviders,
  indexChunks,
  listAiJobs,
  listPromptTemplates,
  loadAiProviderSettings,
  previewChunks,
  saveAiProviderSettings,
  testClaudeCliProvider,
  testCodexCliProvider,
  testOpenAiProvider,
} from '@/api/ai'
import type {
  AiJob,
  AiJobDraft,
  AiProviderConfig,
  AiProviderSettingsDraft,
  AiProviderSettingsView,
  CliProviderTestResult,
  DocumentChunkRecord,
  OpenAiProviderTestResult,
  PromptTemplate,
  TextChunk,
} from '@/types/ai'

interface AiState {
  providers: AiProviderConfig[]
  providerSettings: AiProviderSettingsView[]
  chunks: DocumentChunkRecord[]
  preview: TextChunk[]
  jobs: AiJob[]
  prompts: PromptTemplate[]
  openAiProviderTest: OpenAiProviderTestResult | null
  codexCliProviderTest: CliProviderTestResult | null
  claudeCliProviderTest: CliProviderTestResult | null
  loading: boolean
}

export const useAiStore = defineStore('ai', {
  state: (): AiState => ({
    providers: [],
    providerSettings: [],
    chunks: [],
    preview: [],
    jobs: [],
    prompts: [],
    openAiProviderTest: null,
    codexCliProviderTest: null,
    claudeCliProviderTest: null,
    loading: false,
  }),
  actions: {
    async loadDefaults() {
      this.providers = await defaultAiProviders()
    },
    async loadProviderSettings() {
      this.providerSettings = await loadAiProviderSettings()
    },
    async saveProviderSettings(drafts: AiProviderSettingsDraft[]) {
      this.providerSettings = await saveAiProviderSettings(drafts)
    },
    async testOpenAiProvider() {
      this.openAiProviderTest = await testOpenAiProvider()
      return this.openAiProviderTest
    },
    async testCodexCliProvider() {
      this.codexCliProviderTest = await testCodexCliProvider()
      return this.codexCliProviderTest
    },
    async testClaudeCliProvider() {
      this.claudeCliProviderTest = await testClaudeCliProvider()
      return this.claudeCliProviderTest
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
