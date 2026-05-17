import { callCommand } from './client'
import type {
  AiJob,
  AiJobDraft,
  AiProviderConfig,
  AiProviderSettingsDraft,
  AiProviderSettingsView,
  ContextPack,
  DocumentChunkRecord,
  OpenAiProviderTestResult,
  PromptTemplate,
  TextChunk,
} from '@/types/ai'

export function defaultAiProviders(): Promise<AiProviderConfig[]> {
  return callCommand<AiProviderConfig[]>('default_ai_providers').then((items) => items ?? [])
}

export function loadAiProviderSettings(): Promise<AiProviderSettingsView[]> {
  return callCommand<AiProviderSettingsView[]>('load_ai_provider_settings').then(
    (items) => items ?? [],
  )
}

export function saveAiProviderSettings(
  drafts: AiProviderSettingsDraft[],
): Promise<AiProviderSettingsView[]> {
  return callCommand<AiProviderSettingsView[]>('save_ai_provider_settings', { drafts }).then(
    (items) => items ?? [],
  )
}

export function testOpenAiProvider(): Promise<OpenAiProviderTestResult> {
  return callCommand<OpenAiProviderTestResult>('test_openai_provider')
}

export function previewChunks(text: string, maxChars: number): Promise<TextChunk[]> {
  return callCommand<TextChunk[]>('preview_chunks', { text, maxChars }).then((items) => items ?? [])
}

export function indexChunks(projectId: string, maxChars: number): Promise<DocumentChunkRecord[]> {
  return callCommand<DocumentChunkRecord[]>('index_chunks', { projectId, maxChars }).then(
    (items) => items ?? [],
  )
}

export function previewContextPack(
  projectId: string,
  query: string,
  queryVector: number[],
): Promise<ContextPack> {
  return callCommand<ContextPack>('preview_context_pack', { projectId, query, queryVector })
}

export function listPromptTemplates(): Promise<PromptTemplate[]> {
  return callCommand<PromptTemplate[]>('list_prompt_templates_command').then((items) => items ?? [])
}

export function listAiJobs(): Promise<AiJob[]> {
  return callCommand<AiJob[]>('list_ai_jobs').then((items) => items ?? [])
}

export function createAiJob(draft: AiJobDraft): Promise<AiJob> {
  return callCommand<AiJob>('create_ai_job', { draft })
}
