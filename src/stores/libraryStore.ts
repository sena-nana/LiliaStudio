import { defineStore } from 'pinia'
import * as libraryApi from '@/api/library'
import type {
  Axiom,
  AxiomDraft,
  Character,
  CharacterDraft,
  Entry,
  EntryDraft,
  EventDraft,
  EventParticipantDraft,
  EventRecord,
  Relation,
  RelationDraft,
} from '@/types/library'

interface LibraryState {
  entries: Entry[]
  characters: Character[]
  events: EventRecord[]
  axioms: Axiom[]
  relations: Relation[]
  loading: boolean
  error: string | null
}

export const useLibraryStore = defineStore('library', {
  state: (): LibraryState => ({
    entries: [],
    characters: [],
    events: [],
    axioms: [],
    relations: [],
    loading: false,
    error: null,
  }),
  actions: {
    async loadProject(projectId: string) {
      this.loading = true
      this.error = null
      try {
        const [entries, characters, events, axioms, relations] = await Promise.all([
          libraryApi.listEntries(projectId),
          libraryApi.listCharacters(projectId),
          libraryApi.listEvents(projectId),
          libraryApi.searchAxioms(projectId, ''),
          libraryApi.listRelations(projectId),
        ])
        this.entries = entries
        this.characters = characters
        this.events = events
        this.axioms = axioms
        this.relations = relations
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error)
      } finally {
        this.loading = false
      }
    },
    async createEntry(draft: EntryDraft) {
      const entry = await libraryApi.createEntry(draft)
      prependRecord(this.entries, entry)
      return entry
    },
    async updateEntry(id: string, draft: EntryDraft) {
      const entry = await libraryApi.updateEntry(id, draft)
      replaceRecord(this.entries, entry)
      return entry
    },
    async deleteEntry(id: string) {
      await libraryApi.deleteEntry(id)
      this.entries = removeRecord(this.entries, id)
    },
    async createCharacter(draft: CharacterDraft) {
      const character = await libraryApi.createCharacter(draft)
      prependRecord(this.characters, character)
      return character
    },
    async updateCharacter(id: string, draft: CharacterDraft) {
      const character = await libraryApi.updateCharacter(id, draft)
      replaceRecord(this.characters, character)
      return character
    },
    async deleteCharacter(id: string) {
      await libraryApi.deleteCharacter(id)
      this.characters = removeRecord(this.characters, id)
    },
    async createEvent(draft: EventDraft, participants: EventParticipantDraft[] = []) {
      const event = await libraryApi.createEvent(draft, participants)
      prependRecord(this.events, event)
      return event
    },
    async updateEvent(id: string, draft: EventDraft, participants: EventParticipantDraft[] = []) {
      const event = await libraryApi.updateEvent(id, draft, participants)
      replaceRecord(this.events, event)
      return event
    },
    async deleteEvent(id: string) {
      await libraryApi.deleteEvent(id)
      this.events = removeRecord(this.events, id)
    },
    async createAxiom(draft: AxiomDraft) {
      const axiom = await libraryApi.createAxiom(draft)
      prependRecord(this.axioms, axiom)
      return axiom
    },
    async updateAxiom(id: string, draft: AxiomDraft) {
      const axiom = await libraryApi.updateAxiom(id, draft)
      replaceRecord(this.axioms, axiom)
      return axiom
    },
    async deleteAxiom(id: string) {
      await libraryApi.deleteAxiom(id)
      this.axioms = removeRecord(this.axioms, id)
    },
    async createRelation(draft: RelationDraft) {
      const relation = await libraryApi.createRelation(draft)
      prependRecord(this.relations, relation)
      return relation
    },
    async updateRelation(id: string, draft: RelationDraft) {
      const relation = await libraryApi.updateRelation(id, draft)
      replaceRecord(this.relations, relation)
      return relation
    },
    async deleteRelation(id: string) {
      await libraryApi.deleteRelation(id)
      this.relations = removeRecord(this.relations, id)
    },
  },
})

function prependRecord<T>(items: T[], item: T) {
  items.unshift(item)
}

function replaceRecord<T extends { id: string }>(items: T[], item: T) {
  const index = items.findIndex((current) => current.id === item.id)
  if (index >= 0) {
    items.splice(index, 1, item)
  } else {
    items.unshift(item)
  }
}

function removeRecord<T extends { id: string }>(items: T[], id: string): T[] {
  return items.filter((item) => item.id !== id)
}
