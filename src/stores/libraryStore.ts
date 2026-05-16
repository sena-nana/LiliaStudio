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
        const [entries, characters, events, axioms] = await Promise.all([
          libraryApi.listEntries(projectId),
          libraryApi.listCharacters(projectId),
          libraryApi.listEvents(projectId),
          libraryApi.searchAxioms(projectId, ''),
        ])
        this.entries = entries
        this.characters = characters
        this.events = events
        this.axioms = axioms
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
    async createCharacter(draft: CharacterDraft) {
      const character = await libraryApi.createCharacter(draft)
      this.characters = [character, ...this.characters]
      return character
    },
    async createEvent(draft: EventDraft, participants: EventParticipantDraft[] = []) {
      const event = await libraryApi.createEvent(draft, participants)
      this.events = [event, ...this.events]
      return event
    },
    async createAxiom(draft: AxiomDraft) {
      const axiom = await libraryApi.createAxiom(draft)
      this.axioms = [axiom, ...this.axioms]
      return axiom
    },
    async createRelation(draft: RelationDraft) {
      const relation = await libraryApi.createRelation(draft)
      this.relations = [relation, ...this.relations]
      return relation
    },
  },
})
