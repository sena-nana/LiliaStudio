import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSearchStore } from '@/stores/searchStore'

const invokeMock = vi.mocked(invoke)

describe('searchStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('runs project search through the command API', async () => {
    invokeMock.mockResolvedValueOnce([
      { entityType: 'entry', entityId: 'entry_1', title: '月光阔剑', snippet: '潮汐能', score: 2 },
    ])
    invokeMock.mockResolvedValueOnce({
      status: 'ready',
      message: '',
      model: 'story-embed',
      items: [{ entityType: 'character', entityId: 'char_1', title: '潮汐观测者', snippet: '研究月光阔剑', score: 0.92 }],
    })
    const store = useSearchStore()

    await store.run('project_1', '月光')

    expect(invokeMock).toHaveBeenCalledWith('search_entities', {
      filter: { projectId: 'project_1', query: '月光', entityTypes: [] },
    })
    expect(invokeMock).toHaveBeenCalledWith('search_semantic', {
      request: { projectId: 'project_1', query: '月光', limit: 8 },
    })
    expect(store.results.map((item) => `${item.source}:${item.title}`)).toEqual([
      'keyword:月光阔剑',
      'semantic:潮汐观测者',
    ])
  })

  it('keeps keyword results when semantic search degrades', async () => {
    invokeMock.mockResolvedValueOnce([
      { entityType: 'entry', entityId: 'entry_1', title: '月光阔剑', snippet: '潮汐能', score: 2 },
    ])
    invokeMock.mockResolvedValueOnce({
      status: 'degraded',
      message: '当前项目还没有可用的 embedding 索引。',
      model: '',
      items: [],
    })
    const store = useSearchStore()

    await store.run('project_1', '月光')

    expect(store.results).toHaveLength(1)
    expect(store.semanticStatus).toBe('degraded')
    expect(store.semanticMessage).toContain('embedding')
  })

  it('deduplicates semantic results when keyword results already include the entity', async () => {
    invokeMock.mockResolvedValueOnce([
      { entityType: 'entry', entityId: 'entry_1', title: '月光阔剑', snippet: '潮汐能', score: 2 },
    ])
    invokeMock.mockResolvedValueOnce({
      status: 'ready',
      message: '',
      model: 'story-embed',
      items: [
        { entityType: 'entry', entityId: 'entry_1', title: '月光阔剑', snippet: '潮汐能武器', score: 0.95 },
        { entityType: 'event', entityId: 'event_1', title: '围城战', snippet: '首次公开使用', score: 0.71 },
      ],
    })
    const store = useSearchStore()

    await store.run('project_1', '月光')

    expect(store.results.map((item) => `${item.source}:${item.entityType}:${item.entityId}`)).toEqual([
      'keyword:entry:entry_1',
      'semantic:event:event_1',
    ])
  })
})
