import type { Project } from './project'
import type { Axiom, Character, Entry, EventRecord, Relation } from './library'

export interface ProjectArchive {
  version: number
  project: Project
  entries: Entry[]
  characters: Character[]
  events: EventRecord[]
  axioms: Axiom[]
  relations: Relation[]
}

export interface ImportedProject {
  project: Project
}
