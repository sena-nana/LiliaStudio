import { executeCommand, listResult } from './client'
import { commands } from './commands'
import type {
  CharacterGrowthWorkspaceSnapshot,
  CharacterTraitDeltaRecord,
  CharacterTraitDeltaRecordDraft,
  CharacterTraitState,
  DiagnosticsSummary,
  Fact,
  LogicConflict,
  RepairSuggestion,
  SimulationReport,
  TraitDelta,
} from '@/types/workflows'

export function auditFacts(facts: Fact[]): Promise<LogicConflict[]> {
  return executeCommand(commands.logic.auditFacts, { facts }).then(listResult)
}

export function repairSuggestions(conflict: LogicConflict): Promise<RepairSuggestion[]> {
  return executeCommand(commands.logic.repairSuggestions, { conflict }).then(listResult)
}

export function previewTraitDelta(
  state: CharacterTraitState,
  delta: TraitDelta,
): Promise<CharacterTraitState> {
  return executeCommand(commands.characterGrowth.previewTraitDelta, { state, delta })
}

export function loadCharacterGrowthWorkspace(
  projectId: string,
): Promise<CharacterGrowthWorkspaceSnapshot> {
  return executeCommand(commands.characterGrowth.workspace, { projectId })
}

export function createCharacterTraitDeltaRecord(
  draft: CharacterTraitDeltaRecordDraft,
): Promise<CharacterTraitDeltaRecord> {
  return executeCommand(commands.characterGrowth.createRecord, { draft })
}

export function runSimulation(
  projectId: string,
  scenario: string,
  referencedEntities: string[],
): Promise<SimulationReport> {
  return executeCommand(commands.simulation.run, { projectId, scenario, referencedEntities })
}

export function diagnosticsSummary(): Promise<DiagnosticsSummary> {
  return executeCommand(commands.diagnostics.summary)
}
