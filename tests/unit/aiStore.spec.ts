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

  it('loads prompt templates and AI jobs', async () => {
    invokeMock
      .mockResolvedValueOnce([{ id: 'prompt_1', name: '逻辑审计' }])
      .mockResolvedValueOnce([{ id: 'job_1', status: 'queued' }])

    const store = useAiStore()
    await store.loadPromptsAndJobs()

    expect(store.prompts[0].name).toBe('逻辑审计')
    expect(store.jobs[0].status).toBe('queued')
  })
})
