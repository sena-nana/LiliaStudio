export interface Fact {
  id: string
  subject: string
  predicate: string
  object: string
  scopeTime: string
  scopeLocation: string
}

export interface LogicConflict {
  conflictType: string
  factIds: string[]
  message: string
}

export interface RepairSuggestion {
  title: string
  description: string
  impact: string
}

export interface TraitDelta {
  sourceEventId: string
  traitName: string
  delta: number
  reason: string
}

export interface CharacterTraitState {
  values: Record<string, number>
  sources: TraitDelta[]
}

export interface SimulationReport {
  projectId: string
  scenario: string
  phases: Array<{ label: string; summary: string }>
  risks: string[]
  referencedEntities: string[]
}

export interface DiagnosticsSummary {
  appVersion: string
  platform: string
  database: string
}
