import { callCommand } from './client'
import type {
  CharacterTraitState,
  DiagnosticsSummary,
  Fact,
  LogicConflict,
  RepairSuggestion,
  SimulationReport,
  TraitDelta,
} from '@/types/workflows'

export function auditFacts(facts: Fact[]): Promise<LogicConflict[]> {
  return callCommand<LogicConflict[]>('audit_facts', { facts }).then((items) => items ?? [])
}

export function repairSuggestions(conflict: LogicConflict): Promise<RepairSuggestion[]> {
  return callCommand<RepairSuggestion[]>('repair_suggestions', { conflict }).then((items) => items ?? [])
}

export function previewTraitDelta(
  state: CharacterTraitState,
  delta: TraitDelta,
): Promise<CharacterTraitState> {
  return callCommand<CharacterTraitState>('preview_trait_delta', { state, delta })
}

export function runSimulation(
  projectId: string,
  scenario: string,
  referencedEntities: string[],
): Promise<SimulationReport> {
  return callCommand<SimulationReport>('run_simulation', { projectId, scenario, referencedEntities })
}

export function diagnosticsSummary(): Promise<DiagnosticsSummary> {
  return callCommand<DiagnosticsSummary>('diagnostics_summary')
}
