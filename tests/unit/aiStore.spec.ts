import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAiStore } from '@/stores/aiStore'

const invokeMock = vi.mocked(invoke)

describe('aiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads default providers and indexes chunks through commands', async () => {
    invokeMock
      .mockResolvedValueOnce([{ kind: 'codexCli', commandTemplate: 'codex exec "{prompt}"' }])
      .mockResolvedValueOnce([{ id: 'chunk_1', text: '潮汐能规则' }])

    const store = useAiStore()
    await store.loadDefaults()
    await store.indexProject('project_1', 600)

    expect(store.providers[0].kind).toBe('codexCli')
    expect(store.chunks[0].id).toBe('chunk_1')
    expect(invokeMock).toHaveBeenCalledWith('index_chunks', { projectId: 'project_1', maxChars: 600 })
  })

  it('loads and saves provider settings without exposing raw secrets', async () => {
    invokeMock
      .mockResolvedValueOnce([
        {
          kind: 'openAiCompatible',
          baseUrl: 'https://llm.example/v1',
          apiKeyPreview: 'sk-********1234',
          hasApiKey: true,
          chatModel: 'story-chat',
          embeddingModel: 'story-embed',
          commandTemplate: null,
          enabled: true,
        },
      ])
      .mockResolvedValueOnce([
        {
          kind: 'openAiCompatible',
          baseUrl: 'https://llm.example/v1',
          apiKeyPreview: 'sk-********1234',
          hasApiKey: true,
          chatModel: 'story-chat',
          embeddingModel: 'story-embed',
          commandTemplate: null,
          enabled: true,
        },
      ])

    const store = useAiStore()
    await store.loadProviderSettings()
    await store.saveProviderSettings([
      {
        kind: 'openAiCompatible',
        baseUrl: 'https://llm.example/v1',
        apiKey: 'sk-live-secret-1234',
        clearApiKey: false,
        chatModel: 'story-chat',
        embeddingModel: 'story-embed',
        commandTemplate: null,
        enabled: true,
      },
    ])

    expect(store.providerSettings[0].apiKeyPreview).toBe('sk-********1234')
    expect(invokeMock).toHaveBeenCalledWith('save_ai_provider_settings', {
      drafts: [
        {
          kind: 'openAiCompatible',
          baseUrl: 'https://llm.example/v1',
          apiKey: 'sk-live-secret-1234',
          clearApiKey: false,
          chatModel: 'story-chat',
          embeddingModel: 'story-embed',
          commandTemplate: null,
          enabled: true,
        },
      ],
    })
  })

  it('loads prompt templates and AI jobs', async () => {
    invokeMock
      .mockResolvedValueOnce([{ id: 'prompt_1', name: '逻辑审计' }])
      .mockResolvedValueOnce([{ id: 'job_1', status: 'queued' }])

    const store = useAiStore()
    await store.loadPromptsAndJobs()

    expect(store.prompts[0].name).toBe('逻辑审计')
    expect(store.jobs[0].status).toBe('queued')
  })

  it('stores OpenAI provider test results', async () => {
    invokeMock.mockResolvedValueOnce({
      ok: false,
      message: '请先保存 OpenAI-compatible API Key',
      error: {
        code: 'configMissing',
        message: '请先保存 OpenAI-compatible API Key',
        status: null,
      },
    })

    const store = useAiStore()
    const result = await store.testOpenAiProvider()

    expect(result.ok).toBe(false)
    expect(store.openAiProviderTest?.error?.code).toBe('configMissing')
    expect(invokeMock).toHaveBeenCalledWith('test_openai_provider')
  })

  it('stores Codex CLI provider test results', async () => {
    invokeMock.mockResolvedValueOnce({
      ok: false,
      message: '未找到 Codex CLI，请先安装 codex 并确认 PATH 可用',
      error: {
        code: 'missingCli',
        message: '未找到 Codex CLI，请先安装 codex 并确认 PATH 可用',
        exitCode: null,
      },
      output: null,
    })

    const store = useAiStore()
    const result = await store.testCodexCliProvider()

    expect(result.ok).toBe(false)
    expect(store.codexCliProviderTest?.error?.code).toBe('missingCli')
    expect(invokeMock).toHaveBeenCalledWith('test_codex_cli_provider')
  })
})
