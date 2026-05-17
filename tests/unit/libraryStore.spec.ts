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
      .mockResolvedValueOnce([{ id: 'relation_1', projectId: 'project_1', relationType: 'derived_from' }])

    const store = useLibraryStore()
    await store.loadProject('project_1')

    expect(store.entries).toHaveLength(1)
    expect(store.characters).toHaveLength(1)
    expect(store.events).toHaveLength(1)
    expect(store.axioms).toHaveLength(1)
    expect(store.relations).toHaveLength(1)
    expect(invokeMock).toHaveBeenCalledWith('list_entries', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('list_characters', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('list_events', { projectId: 'project_1' })
    expect(invokeMock).toHaveBeenCalledWith('search_axioms', { projectId: 'project_1', query: '' })
    expect(invokeMock).toHaveBeenCalledWith('list_relations', { projectId: 'project_1' })
  })

  it('updates and removes library records through commands', async () => {
    const store = useLibraryStore()
    store.entries = [{ id: 'entry_1', projectId: 'project_1', title: '旧词条' } as any]
    store.characters = [{ id: 'character_1', projectId: 'project_1', name: '旧角色' } as any]
    store.events = [{ id: 'event_1', projectId: 'project_1', title: '旧事件' } as any]
    store.axioms = [{ id: 'axiom_1', projectId: 'project_1', subject: '旧公理' } as any]
    store.relations = [{ id: 'relation_1', projectId: 'project_1', relationType: 'old' } as any]

    invokeMock
      .mockResolvedValueOnce({ id: 'entry_1', projectId: 'project_1', title: '新词条' })
      .mockResolvedValueOnce({ id: 'character_1', projectId: 'project_1', name: '新角色' })
      .mockResolvedValueOnce({ id: 'event_1', projectId: 'project_1', title: '新事件' })
      .mockResolvedValueOnce({ id: 'axiom_1', projectId: 'project_1', subject: '新公理' })
      .mockResolvedValueOnce({ id: 'relation_1', projectId: 'project_1', relationType: 'new' })
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)

    await store.updateEntry('entry_1', {
      projectId: 'project_1',
      entryType: 'item',
      title: '新词条',
      summary: '',
      body: '',
      tags: [],
      status: 'draft',
    })
    await store.updateCharacter('character_1', {
      projectId: 'project_1',
      name: '新角色',
      aliases: [],
      summary: '',
      appearance: '',
      goals: '',
      motivations: '',
      fears: '',
      faction: '',
      tags: [],
    })
    await store.updateEvent('event_1', {
      projectId: 'project_1',
      title: '新事件',
      description: '',
      timeLabel: '',
      sortKey: 1,
      startLabel: '',
      endLabel: '',
      location: '',
      importance: 1,
      outcome: '',
      tags: [],
    })
    await store.updateAxiom('axiom_1', {
      projectId: 'project_1',
      subject: '新公理',
      predicate: 'defines',
      object: '值',
      scopeTime: '',
      scopeLocation: '',
      certainty: 1,
      sourceEntityType: null,
      sourceEntityId: null,
      naturalLanguage: '',
      tags: [],
    })
    await store.updateRelation('relation_1', {
      projectId: 'project_1',
      source: { entityType: 'entry', entityId: 'entry_1' },
      target: { entityType: 'axiom', entityId: 'axiom_1' },
      relationType: 'new',
      description: '',
      confidence: 1,
      directed: true,
    })

    expect(store.entries[0].title).toBe('新词条')
    expect(store.characters[0].name).toBe('新角色')
    expect(store.events[0].title).toBe('新事件')
    expect(store.axioms[0].subject).toBe('新公理')
    expect(store.relations[0].relationType).toBe('new')

    await store.deleteEntry('entry_1')
    await store.deleteCharacter('character_1')
    await store.deleteEvent('event_1')
    await store.deleteAxiom('axiom_1')
    await store.deleteRelation('relation_1')

    expect(store.entries).toHaveLength(0)
    expect(store.characters).toHaveLength(0)
    expect(store.events).toHaveLength(0)
    expect(store.axioms).toHaveLength(0)
    expect(store.relations).toHaveLength(0)
    expect(invokeMock).toHaveBeenCalledWith('delete_relation', { id: 'relation_1' })
  })
})
