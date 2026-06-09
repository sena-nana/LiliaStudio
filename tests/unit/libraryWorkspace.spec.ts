import { describe, expect, it } from 'vitest'
import {
  buildEntityOptions,
  characterToDraft,
  entryToDraft,
  entityLabel,
  makeEntityKey,
  parseEntityKey,
  parseList,
} from '@/domain/libraryWorkspace'
import type { Character, Entry } from '@/types/library'

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
})
