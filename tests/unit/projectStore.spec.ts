import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProjectStore } from '@/stores/projectStore'

const invokeMock = vi.mocked(invoke)

describe('projectStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
  })

  it('loads and creates projects through the API layer', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 'project_1',
        name: '雨夜都市',
        description: '城市设定',
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
        archivedAt: null,
      },
    ])
    const store = useProjectStore()

    await store.loadProjects()

    expect(store.projects).toHaveLength(1)
    expect(store.projects[0].name).toBe('雨夜都市')

    invokeMock.mockResolvedValueOnce({
      id: 'project_2',
      name: '北境',
      description: '',
      createdAt: '2026-01-02T00:00:00Z',
      updatedAt: '2026-01-02T00:00:00Z',
      archivedAt: null,
    })

    await store.createProject({ name: '北境', description: '' })

    expect(invokeMock).toHaveBeenLastCalledWith('create_project', {
      draft: { name: '北境', description: '' },
    })
    expect(store.projects.map((project) => project.name)).toEqual(['北境', '雨夜都市'])
  })
})
