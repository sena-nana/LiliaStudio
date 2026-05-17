export type AiProviderKind = 'openAiCompatible' | 'codexCli' | 'claudeCli'

export interface AiProviderConfig {
  kind: AiProviderKind
  baseUrl: string | null
  apiKey: string | null
  chatModel: string | null
  embeddingModel: string | null
  commandTemplate: string | null
  enabled: boolean
}

export interface AiProviderSettingsView {
  kind: AiProviderKind
  baseUrl: string | null
  apiKeyPreview: string | null
  hasApiKey: boolean
  chatModel: string | null
  embeddingModel: string | null
  commandTemplate: string | null
  enabled: boolean
}

export interface AiProviderSettingsDraft {
  kind: AiProviderKind
  baseUrl: string | null
  apiKey: string | null
  clearApiKey: boolean
  chatModel: string | null
  embeddingModel: string | null
  commandTemplate: string | null
  enabled: boolean
}

export type ProviderErrorCode =
  | 'configMissing'
  | 'authFailed'
  | 'httpError'
  | 'networkError'
  | 'modelResponseInvalid'

export interface ProviderError {
  code: ProviderErrorCode
  message: string
  status: number | null
}

export interface OpenAiProviderTestResult {
  ok: boolean
  message: string
  error: ProviderError | null
}

export type CliProviderErrorCode = 'missingCli' | 'authFailed' | 'executionFailed' | 'timedOut'

export interface CliProviderError {
  code: CliProviderErrorCode
  message: string
  exitCode: number | null
}

export interface CliProviderTestResult {
  ok: boolean
  message: string
  error: CliProviderError | null
  output: string | null
}

export type AiJobStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled'

export type AiJobLogLevel = 'info' | 'warning' | 'error'

export interface AiJobLog {
  id: string
  jobId: string
  level: AiJobLogLevel
  message: string
  createdAt: string
}

export interface TextChunk {
  ordinal: number
  text: string
  contentHash: string
}

export interface DocumentChunkRecord {
  id: string
  projectId: string
  sourceType: string
  sourceId: string
  ordinal: number
  text: string
  contentHash: string
}

export interface ContextPack {
  projectId: string
  query: string
  items: Array<{
    sourceType: string
    sourceId: string
    text: string
    score: number
  }>
}

export interface AiJob {
  id: string
  projectId: string | null
  providerKind: string
  jobType: string
  status: AiJobStatus
  inputSummary: string
  outputText: string
  errorMessage: string | null
  startedAt: string | null
  finishedAt: string | null
  cancelRequestedAt: string | null
  retryOfJobId: string | null
  createdAt: string
  updatedAt: string
}

export interface AiJobDraft {
  projectId: string | null
  providerKind: string
  jobType: string
  inputSummary: string
}

export interface PromptTemplate {
  id: string
  name: string
  purpose: string
  template: string
  builtIn: boolean
  createdAt: string
  updatedAt: string
}
