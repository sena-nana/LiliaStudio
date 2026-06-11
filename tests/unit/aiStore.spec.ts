import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAiStore } from '@/stores/aiStore'

const invokeMock = vi.mocked(invoke)

function makeOpenAiProviderSettings(overrides: Record<string, unknown> = {}) {
  return {
    kind: 'openAiCompatible',
    baseUrl: 'https://llm.example/v1',
    apiKeyPreview: 'sk-********1234',
    hasApiKey: true,
    chatModel: 'story-chat',
    embeddingModel: 'story-embed',
    commandTemplate: null,
    enabled: true,
    ...overrides,
  }
}

function makeChunk(overrides: Record<string, unknown> = {}) {
  return {
    id: 'chunk_1',
    projectId: 'project_1',
    sourceType: 'character',
    sourceId: 'character_1',
    ordinal: 0,
    text: '潮汐能规则',
    contentHash: 'hash_1',
    estimatedTokens: 3,
    updatedAt: '2026-06-10T00:00:00Z',
    ...overrides,
  }
}

function makeIndexStatus(overrides: Record<string, unknown> = {}) {
  return {
    chunkCount: 0,
    embeddingCount: 0,
    missingEmbeddingCount: 0,
    staleEmbeddingCount: 0,
    model: '',
    status: 'degraded',
    message: '',
    ...overrides,
  }
}

describe('aiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads default providers and indexes chunks through commands', async () => {
    invokeMock
      .mockResolvedValueOnce([{ kind: 'codexCli', commandTemplate: 'codex exec "{prompt}"' }])
      .mockResolvedValueOnce([
        {
          id: 'chunk_1',
          projectId: 'project_1',
          sourceType: 'character',
          sourceId: 'character_1',
          ordinal: 0,
          text: '潮汐能规则',
          contentHash: 'hash_1',
          estimatedTokens: 3,
          updatedAt: '2026-06-10T00:00:00Z',
        },
      ])

    const store = useAiStore()
    await store.loadDefaults()
    await store.indexProject('project_1', 600)

    expect(store.providers[0].kind).toBe('codexCli')
    expect(store.chunks[0].id).toBe('chunk_1')
    expect(store.chunks[0].sourceType).toBe('character')
    expect(store.chunks[0].estimatedTokens).toBe(3)
    expect(store.chunks[0].updatedAt).toBe('2026-06-10T00:00:00Z')
    expect(invokeMock).toHaveBeenCalledWith('rag_index_chunks', { projectId: 'project_1', maxChars: 600 })
  })

  it('rebuilds embeddings when provider settings are ready', async () => {
    invokeMock
      .mockResolvedValueOnce([makeOpenAiProviderSettings()])
      .mockResolvedValueOnce({
        chunkCount: 12,
        embeddingCount: 12,
        model: 'story-embed',
      })
      .mockResolvedValueOnce(
        makeIndexStatus({
          chunkCount: 12,
          embeddingCount: 12,
          model: 'story-embed',
          status: 'ready',
        }),
      )

    const store = useAiStore()
    await store.rebuildEmbeddingIndex('project_1', 600)

    expect(store.indexState).toMatchObject({
      chunkCount: 12,
      embeddingCount: 12,
      missingEmbeddingCount: 0,
      staleEmbeddingCount: 0,
      model: 'story-embed',
      status: 'ready',
      message: '',
      lastProjectId: 'project_1',
    })
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'ai_load_provider_settings')
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'rag_index_embeddings', { projectId: 'project_1', maxChars: 600 })
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'rag_index_status', { projectId: 'project_1' })
  })

  it('falls back to chunk indexing when embedding provider is not configured', async () => {
    invokeMock
      .mockResolvedValueOnce([
        makeOpenAiProviderSettings({
          apiKeyPreview: null,
          hasApiKey: false,
        }),
      ])
      .mockResolvedValueOnce([makeChunk()])
      .mockResolvedValueOnce(
        makeIndexStatus({
          chunkCount: 1,
          embeddingCount: 0,
          missingEmbeddingCount: 1,
          model: '',
          status: 'degraded',
          message: '请先启用 OpenAI 兼容接口并填写嵌入模型。',
        }),
      )

    const store = useAiStore()
    await store.rebuildEmbeddingIndex('project_1', 600)

    expect(store.indexState).toMatchObject({
      chunkCount: 1,
      embeddingCount: 0,
      missingEmbeddingCount: 1,
      staleEmbeddingCount: 0,
      model: '',
      status: 'degraded',
      message: '请先启用 OpenAI 兼容接口并填写嵌入模型。',
      lastProjectId: 'project_1',
    })
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'rag_index_chunks', { projectId: 'project_1', maxChars: 600 })
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'rag_index_status', { projectId: 'project_1' })
    expect(invokeMock).not.toHaveBeenCalledWith('rag_index_embeddings', expect.anything())
  })

  it('stores failed embedding indexing without clearing previous counters', async () => {
    invokeMock
      .mockResolvedValueOnce([makeOpenAiProviderSettings()])
      .mockResolvedValueOnce({
        chunkCount: 8,
        embeddingCount: 8,
        model: 'story-embed',
      })
      .mockResolvedValueOnce(
        makeIndexStatus({
          chunkCount: 8,
          embeddingCount: 8,
          model: 'story-embed',
          status: 'ready',
        }),
      )

    const store = useAiStore()
    await store.rebuildEmbeddingIndex('project_1', 600)

    invokeMock.mockReset()
    invokeMock
      .mockRejectedValueOnce(new Error('provider returned HTTP 401'))

    await store.rebuildEmbeddingIndex('project_1', 600)

    expect(store.indexState).toMatchObject({
      chunkCount: 8,
      embeddingCount: 8,
      model: 'story-embed',
      status: 'failed',
      message: 'provider returned HTTP 401',
      lastProjectId: 'project_1',
    })
  })

  it('loads embedding index status directly', async () => {
    invokeMock.mockResolvedValueOnce(
      makeIndexStatus({
        chunkCount: 6,
        embeddingCount: 4,
        missingEmbeddingCount: 1,
        staleEmbeddingCount: 1,
        model: 'story-embed',
        status: 'degraded',
        message: '当前 embedding 索引不完整或已过期。',
      }),
    )

    const store = useAiStore()
    await store.loadIndexStatus('project_1')

    expect(store.indexState).toMatchObject({
      chunkCount: 6,
      embeddingCount: 4,
      missingEmbeddingCount: 1,
      staleEmbeddingCount: 1,
      model: 'story-embed',
      status: 'degraded',
      message: '当前 embedding 索引不完整或已过期。',
      lastProjectId: 'project_1',
    })
    expect(invokeMock).toHaveBeenCalledWith('rag_index_status', { projectId: 'project_1' })
  })

  it('loads and saves provider settings without exposing raw secrets', async () => {
    invokeMock
      .mockResolvedValueOnce([makeOpenAiProviderSettings()])
      .mockResolvedValueOnce([makeOpenAiProviderSettings()])

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
    expect(invokeMock).toHaveBeenCalledWith('ai_save_provider_settings', {
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
      message: '请先保存 OpenAI 兼容接口的接口密钥',
      error: {
        code: 'configMissing',
        message: '请先保存 OpenAI 兼容接口的接口密钥',
        status: null,
      },
    })

    const store = useAiStore()
    const result = await store.testOpenAiProvider()

    expect(result.ok).toBe(false)
    expect(store.openAiProviderTest?.error?.code).toBe('configMissing')
    expect(invokeMock).toHaveBeenCalledWith('ai_test_openai_provider')
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
    expect(invokeMock).toHaveBeenCalledWith('ai_test_codex_cli_provider')
  })

  it('stores Claude CLI provider test results', async () => {
    invokeMock.mockResolvedValueOnce({
      ok: false,
      message: '未找到 Claude CLI，请先安装 claude 并确认 PATH 可用',
      error: {
        code: 'missingCli',
        message: '未找到 Claude CLI，请先安装 claude 并确认 PATH 可用',
        exitCode: null,
      },
      output: null,
    })

    const store = useAiStore()
    const result = await store.testClaudeCliProvider()

    expect(result.ok).toBe(false)
    expect(store.claudeCliProviderTest?.error?.code).toBe('missingCli')
    expect(invokeMock).toHaveBeenCalledWith('ai_test_claude_cli_provider')
  })
})
