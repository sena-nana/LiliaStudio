import { callCommand } from './client'
import type {
  Axiom,
  AxiomDraft,
  Character,
  CharacterDraft,
  Entry,
  EntryDraft,
  EntityRef,
  EventDraft,
  EventParticipantDraft,
  EventRecord,
  Relation,
  RelationDraft,
} from '@/types/library'

export function listEntries(projectId: string): Promise<Entry[]> {
  return callCommand<Entry[]>('list_entries', { projectId }).then((items) => items ?? [])
}

export function createEntry(draft: EntryDraft): Promise<Entry> {
  return callCommand<Entry>('create_entry', { draft })
}

export function listCharacters(projectId: string): Promise<Character[]> {
  return callCommand<Character[]>('list_characters', { projectId }).then((items) => items ?? [])
}

export function createCharacter(draft: CharacterDraft): Promise<Character> {
  return callCommand<Character>('create_character', { draft })
}

export function listEvents(projectId: string): Promise<EventRecord[]> {
  return callCommand<EventRecord[]>('list_events', { projectId }).then((items) => items ?? [])
}

export function createEvent(
  draft: EventDraft,
  participants: EventParticipantDraft[] = [],
): Promise<EventRecord> {
  return callCommand<EventRecord>('create_event', { draft, participants })
}

export function searchAxioms(projectId: string, query: string): Promise<Axiom[]> {
  return callCommand<Axiom[]>('search_axioms', { projectId, query }).then((items) => items ?? [])
}

export function createAxiom(draft: AxiomDraft): Promise<Axiom> {
  return callCommand<Axiom>('create_axiom', { draft })
}

export function listBacklinks(target: EntityRef): Promise<Relation[]> {
  return callCommand<Relation[]>('list_backlinks', { target }).then((items) => items ?? [])
}

export function createRelation(draft: RelationDraft): Promise<Relation> {
  return callCommand<Relation>('create_relation', { draft })
}
