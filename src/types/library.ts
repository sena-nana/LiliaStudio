export interface BaseEntity {
  id: string
  projectId: string
  createdAt: string
  updatedAt: string
  deletedAt: string | null
}

export interface Entry extends BaseEntity {
  entryType: string
  title: string
  summary: string
  body: string
  tags: string[]
  status: string
}

export interface EntryDraft {
  projectId: string
  entryType: string
  title: string
  summary: string
  body: string
  tags: string[]
  status: string
}

export interface Character extends BaseEntity {
  name: string
  aliases: string[]
  summary: string
  appearance: string
  goals: string
  motivations: string
  fears: string
  faction: string
  tags: string[]
}

export interface CharacterDraft {
  projectId: string
  name: string
  aliases: string[]
  summary: string
  appearance: string
  goals: string
  motivations: string
  fears: string
  faction: string
  tags: string[]
}

export interface EventRecord extends BaseEntity {
  title: string
  description: string
  timeLabel: string
  sortKey: number
  startLabel: string
  endLabel: string
  location: string
  importance: number
  outcome: string
  tags: string[]
}

export interface EventDraft {
  projectId: string
  title: string
  description: string
  timeLabel: string
  sortKey: number
  startLabel: string
  endLabel: string
  location: string
  importance: number
  outcome: string
  tags: string[]
}

export interface EventParticipantDraft {
  entityType: string
  entityId: string
  role: string
}

export interface Axiom extends BaseEntity {
  subject: string
  predicate: string
  object: string
  scopeTime: string
  scopeLocation: string
  certainty: number
  sourceEntityType: string | null
  sourceEntityId: string | null
  naturalLanguage: string
  tags: string[]
}

export interface AxiomDraft {
  projectId: string
  subject: string
  predicate: string
  object: string
  scopeTime: string
  scopeLocation: string
  certainty: number
  sourceEntityType: string | null
  sourceEntityId: string | null
  naturalLanguage: string
  tags: string[]
}

export interface EntityRef {
  entityType: string
  entityId: string
}

export interface Relation {
  id: string
  projectId: string
  source: EntityRef
  target: EntityRef
  relationType: string
  description: string
  confidence: number
  directed: boolean
  createdAt: string
  updatedAt: string
  deletedAt: string | null
}

export interface RelationDraft {
  projectId: string
  source: EntityRef
  target: EntityRef
  relationType: string
  description: string
  confidence: number
  directed: boolean
}
