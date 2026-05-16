import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLibraryStore } from '@/stores/libraryStore'

const invokeMock = vi.mocked(invoke)

describe('libraryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
    invokeMock.mockResolvedValue([])
  })

  it('loads the local project library collections', async () => {
    invokeMock
      .mockResolvedValueOnce([{ id: 'entry_1', projectId: 'project_1', title: '月光阔剑' }])
      .mockResolvedValueOnce([{ id: 'character_1', projectId: 'project_1', name: '椎名' }])
      .mockResolvedValueOnce([{ id: 'event_1', projectId: 'project_1', title: '围城战' }])
      .mockResolvedValueOnce([{ id: 'axiom_1', projectId: 'project_1', subject: '月光金属' }])

    const store = useLibraryStore()
    await store.loadProject('project_1')

    expect(store.entries).toHaveLength(1)
    expect(store.characters).toHaveLength(1)
    expect(store.events).toHaveLength(1)
    expect(store.axioms).toHaveLength(1)
    expect(invokeMock).toHaveBeenCalledWith('list_entries', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('list_characters', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('list_events', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('search_axioms', { projectId: 'project_1', query: '' })
  })
})
