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

  it('updates and archives projects through the API layer', async () => {
    const store = useProjectStore()
    store.projects = [
      {
        id: 'project_1',
        name: '旧项目',
        description: '',
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
        archivedAt: null,
      },
    ]
    store.activeProjectId = 'project_1'
    invokeMock
      .mockResolvedValueOnce({
        id: 'project_1',
        name: '新项目',
        description: '更新后的描述',
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-02T00:00:00Z',
        archivedAt: null,
      })
      .mockResolvedValueOnce(undefined)

    await store.updateProject('project_1', {
      name: '新项目',
      description: '更新后的描述',
    })
    await store.archiveProject('project_1')

    expect(invokeMock).toHaveBeenCalledWith('update_project', {
      id: 'project_1',
      draft: { name: '新项目', description: '更新后的描述' },
    })
    expect(invokeMock).toHaveBeenCalledWith('archive_project', { id: 'project_1' })
    expect(store.projects).toHaveLength(0)
    expect(store.activeProjectId).toBeNull()
  })
})
