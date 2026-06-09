import { commands } from './commands'
import { executeCommand, listResult } from './client'
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
  LibraryProjectSnapshot,
  Relation,
  RelationDraft,
} from '@/types/library'

export function loadProjectSnapshot(projectId: string): Promise<LibraryProjectSnapshot> {
  return executeCommand(commands.library.projectSnapshot, { projectId })
}

export function createEntry(draft: EntryDraft): Promise<Entry> {
  return executeCommand(commands.library.createEntry, { draft })
}

export function updateEntry(id: string, draft: EntryDraft): Promise<Entry> {
  return executeCommand(commands.library.updateEntry, { id, draft })
}

export function deleteEntry(id: string): Promise<void> {
  return executeCommand(commands.library.deleteEntry, { id })
}

export function createCharacter(draft: CharacterDraft): Promise<Character> {
  return executeCommand(commands.library.createCharacter, { draft })
}

export function updateCharacter(id: string, draft: CharacterDraft): Promise<Character> {
  return executeCommand(commands.library.updateCharacter, { id, draft })
}

export function deleteCharacter(id: string): Promise<void> {
  return executeCommand(commands.library.deleteCharacter, { id })
}

export function createEvent(
  draft: EventDraft,
  participants: EventParticipantDraft[] = [],
): Promise<EventRecord> {
  return executeCommand(commands.library.createEvent, { draft, participants })
}

export function updateEvent(
  id: string,
  draft: EventDraft,
  participants: EventParticipantDraft[] = [],
): Promise<EventRecord> {
  return executeCommand(commands.library.updateEvent, { id, draft, participants })
}

export function deleteEvent(id: string): Promise<void> {
  return executeCommand(commands.library.deleteEvent, { id })
}

export function createAxiom(draft: AxiomDraft): Promise<Axiom> {
  return executeCommand(commands.library.createAxiom, { draft })
}

export function updateAxiom(id: string, draft: AxiomDraft): Promise<Axiom> {
  return executeCommand(commands.library.updateAxiom, { id, draft })
}

export function deleteAxiom(id: string): Promise<void> {
  return executeCommand(commands.library.deleteAxiom, { id })
}

export function listBacklinks(target: EntityRef): Promise<Relation[]> {
  return executeCommand(commands.library.listBacklinks, { target }).then(listResult)
}

export function createRelation(draft: RelationDraft): Promise<Relation> {
  return executeCommand(commands.library.createRelation, { draft })
}

export function updateRelation(id: string, draft: RelationDraft): Promise<Relation> {
  return executeCommand(commands.library.updateRelation, { id, draft })
}

export function deleteRelation(id: string): Promise<void> {
  return executeCommand(commands.library.deleteRelation, { id })
}
