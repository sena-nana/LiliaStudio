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
      this.entries = [entry, ...this.entries]
      return entry
    },
    async updateEntry(id: string, draft: EntryDraft) {
      const entry = await libraryApi.updateEntry(id, draft)
      this.replaceById(this.entries, entry)
      return entry
    },
    async deleteEntry(id: string) {
      await libraryApi.deleteEntry(id)
      this.entries = this.entries.filter((entry) => entry.id !== id)
    },
    async createCharacter(draft: CharacterDraft) {
      const character = await libraryApi.createCharacter(draft)
      this.characters = [character, ...this.characters]
      return character
    },
    async updateCharacter(id: string, draft: CharacterDraft) {
      const character = await libraryApi.updateCharacter(id, draft)
      this.replaceById(this.characters, character)
      return character
    },
    async deleteCharacter(id: string) {
      await libraryApi.deleteCharacter(id)
      this.characters = this.characters.filter((character) => character.id !== id)
    },
    async createEvent(draft: EventDraft, participants: EventParticipantDraft[] = []) {
      const event = await libraryApi.createEvent(draft, participants)
      this.events = [event, ...this.events]
      return event
    },
    async updateEvent(id: string, draft: EventDraft, participants: EventParticipantDraft[] = []) {
      const event = await libraryApi.updateEvent(id, draft, participants)
      this.replaceById(this.events, event)
      return event
    },
    async deleteEvent(id: string) {
      await libraryApi.deleteEvent(id)
      this.events = this.events.filter((event) => event.id !== id)
    },
    async createAxiom(draft: AxiomDraft) {
      const axiom = await libraryApi.createAxiom(draft)
      this.axioms = [axiom, ...this.axioms]
      return axiom
    },
    async updateAxiom(id: string, draft: AxiomDraft) {
      const axiom = await libraryApi.updateAxiom(id, draft)
      this.replaceById(this.axioms, axiom)
      return axiom
    },
    async deleteAxiom(id: string) {
      await libraryApi.deleteAxiom(id)
      this.axioms = this.axioms.filter((axiom) => axiom.id !== id)
    },
    async createRelation(draft: RelationDraft) {
      const relation = await libraryApi.createRelation(draft)
      this.relations = [relation, ...this.relations]
      return relation
    },
    async updateRelation(id: string, draft: RelationDraft) {
      const relation = await libraryApi.updateRelation(id, draft)
      this.replaceById(this.relations, relation)
      return relation
    },
    async deleteRelation(id: string) {
      await libraryApi.deleteRelation(id)
      this.relations = this.relations.filter((relation) => relation.id !== id)
    },
    replaceById<T extends { id: string }>(items: T[], item: T) {
      const index = items.findIndex((current) => current.id === item.id)
      if (index >= 0) {
        items.splice(index, 1, item)
      } else {
        items.unshift(item)
      }
    },
  },
})
