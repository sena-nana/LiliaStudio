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
    const store = useSearchStore()

    await store.run('project_1', '月光')

    expect(invokeMock).toHaveBeenCalledWith('search_entities', {
      filter: { projectId: 'project_1', query: '月光', entityTypes: [] },
    })
    expect(store.results[0].title).toBe('月光阔剑')
  })
})
