import { defineStore } from 'pinia'
import {
  defaultAiProviders,
  indexEmbeddings,
  indexChunks,
  loadAiProviderSettings,
  previewChunks,
  saveAiProviderSettings,
  testClaudeCliProvider,
  testCodexCliProvider,
  testOpenAiProvider,
} from '@/api/ai'
import { createAiJob, listAiJobs } from '@/api/jobs'
import { listPromptTemplates } from '@/api/promptTemplates'
import type {
  AiJob,
  AiJobDraft,
  AiProviderConfig,
  AiProviderSettingsDraft,
  AiProviderSettingsView,
  CliProviderTestResult,
  DocumentChunkRecord,
  EmbeddingIndexState,
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
  indexState: EmbeddingIndexState
  loading: boolean
}

const EMBEDDING_CONFIG_MESSAGE = '请先在 AI 设置中启用并配置 OpenAI 兼容接口的嵌入模型和密钥。'

function createIndexState(): EmbeddingIndexState {
  return {
    chunkCount: 0,
    embeddingCount: 0,
    model: '',
    status: 'idle',
    message: '',
    lastProjectId: null,
  }
}

function isProviderReadyForEmbeddings(provider: AiProviderSettingsView | undefined): boolean {
  if (!provider) return false
  return Boolean(
    provider.enabled &&
      provider.hasApiKey &&
      provider.baseUrl?.trim() &&
      provider.embeddingModel?.trim(),
  )
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
    indexState: createIndexState(),
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
    async rebuildEmbeddingIndex(projectId: string, maxChars = 600) {
      if (this.providerSettings.length === 0) {
        await this.loadProviderSettings()
      }
      const previousState = this.indexState
      const currentState =
        previousState.lastProjectId === projectId
          ? { ...previousState }
          : { ...createIndexState(), lastProjectId: projectId }
      this.indexState = {
        ...currentState,
        status: 'loading',
        message: '',
        lastProjectId: projectId,
      }
      this.loading = true
      try {
        const provider = this.providerSettings.find((item) => item.kind === 'openAiCompatible')
        if (!isProviderReadyForEmbeddings(provider)) {
          this.chunks = await indexChunks(projectId, maxChars)
          this.indexState = {
            chunkCount: this.chunks.length,
            embeddingCount: 0,
            model: '',
            status: 'degraded',
            message: EMBEDDING_CONFIG_MESSAGE,
            lastProjectId: projectId,
          }
          return this.indexState
        }
        const result = await indexEmbeddings(projectId, maxChars)
        this.indexState = {
          chunkCount: result.chunkCount,
          embeddingCount: result.embeddingCount,
          model: result.model,
          status: 'ready',
          message: '',
          lastProjectId: projectId,
        }
        return this.indexState
      } catch (error) {
        this.indexState = {
          ...currentState,
          status: 'failed',
          message: error instanceof Error ? error.message : String(error),
          lastProjectId: projectId,
        }
        return this.indexState
      } finally {
        this.loading = false
      }
    },
  },
})
