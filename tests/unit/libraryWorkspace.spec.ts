import { describe, expect, it } from 'vitest'
import {
  buildEntityOptions,
  characterToDraft,
  entryToDraft,
  entityLabel,
  makeEntityKey,
  parseEntityKey,
  parseList,
  relationCountForEntity,
  relationSuggestionToDraft,
  selectionToEntityRef,
} from '@/domain/libraryWorkspace'
import type { Character, Entry, Relation, RelationSuggestion } from '@/types/library'

describe('libraryWorkspace helpers', () => {
  it('parses comma separated lists with Chinese and ASCII commas', () => {
    expect(parseList('月光, 城市， 规则 ,,')).toEqual(['月光', '城市', '规则'])
  })

  it('encodes and decodes entity keys with an entry fallback', () => {
    expect(makeEntityKey('character', 'character_1')).toBe('character:character_1')
    expect(parseEntityKey('character:character_1')).toEqual({
      entityType: 'character',
      entityId: 'character_1',
    })
    expect(parseEntityKey('entry_1')).toEqual({
      entityType: 'entry',
      entityId: 'entry_1',
    })
  })

  it('builds entity options for selectable library records', () => {
    const options = buildEntityOptions({
      entries: [{ id: 'entry_1', title: '月光阔剑' } as Entry],
      characters: [{ id: 'character_1', name: '椎名' } as Character],
      events: [{ id: 'event_1', title: '围城战' } as any],
      axioms: [{ id: 'axiom_1', subject: '月光金属' } as any],
    })

    expect(options).toEqual([
      { key: 'entry:entry_1', label: '词条：月光阔剑' },
      { key: 'character:character_1', label: '角色：椎名' },
      { key: 'event:event_1', label: '事件：围城战' },
      { key: 'axiom:axiom_1', label: '公理：月光金属' },
    ])
    expect(entityLabel({ entityType: 'entry', entityId: 'missing' }, options)).toBe('词条：missing')
  })

  it('copies record arrays when syncing records into drafts', () => {
    const entry = {
      id: 'entry_1',
      projectId: 'project_1',
      entryType: 'item',
      title: '月光阔剑',
      summary: '',
      body: '',
      tags: ['武器'],
      status: 'draft',
      createdAt: '',
      updatedAt: '',
      deletedAt: null,
    }
    const character = {
      id: 'character_1',
      projectId: 'project_1',
      name: '椎名',
      aliases: ['队长'],
      summary: '',
      appearance: '',
      goals: '',
      motivations: '',
      fears: '',
      faction: '',
      tags: ['主角'],
      createdAt: '',
      updatedAt: '',
      deletedAt: null,
    }

    const entryDraft = entryToDraft(entry)
    const characterDraft = characterToDraft(character)

    expect(entryDraft.tags).toEqual(['武器'])
    expect(entryDraft.tags).not.toBe(entry.tags)
    expect(characterDraft.aliases).toEqual(['队长'])
    expect(characterDraft.aliases).not.toBe(character.aliases)
    expect(characterDraft.tags).not.toBe(character.tags)
  })

  it('counts relations touching an entity and ignores relation selections as entity refs', () => {
    const relations: Relation[] = [
      {
        id: 'relation_1',
        projectId: 'project_1',
        source: { entityType: 'character', entityId: 'character_1' },
        target: { entityType: 'event', entityId: 'event_1' },
        relationType: '参与事件',
        description: '',
        confidence: 1,
        directed: true,
        createdAt: '',
        updatedAt: '',
        deletedAt: null,
      },
    ]

    expect(relationCountForEntity(relations, { entityType: 'character', entityId: 'character_1' })).toBe(1)
    expect(selectionToEntityRef({ kind: 'character', id: 'character_1' })).toEqual({
      entityType: 'character',
      entityId: 'character_1',
    })
    expect(selectionToEntityRef({ kind: 'relation', id: 'relation_1' })).toBeNull()
  })

  it('converts a relation suggestion into a confirmable relation draft', () => {
    const suggestion: RelationSuggestion = {
      source: { entityType: 'character', entityId: 'character_1' },
      target: { entityType: 'event', entityId: 'event_1' },
      relationType: '参与事件',
      description: '同一事件参与者',
      confidence: 0.95,
      directed: true,
      reason: '同一事件参与者',
      strength: '高',
    }

    expect(relationSuggestionToDraft('project_1', suggestion)).toEqual({
      projectId: 'project_1',
      source: { entityType: 'character', entityId: 'character_1' },
      target: { entityType: 'event', entityId: 'event_1' },
      relationType: '参与事件',
      description: '同一事件参与者',
      confidence: 0.95,
      directed: true,
    })
  })
})
