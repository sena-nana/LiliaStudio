import { defineStore } from 'pinia'
import { auditFacts, diagnosticsSummary, repairSuggestions, runSimulation } from '@/api/workflows'
import type { DiagnosticsSummary, LogicConflict, RepairSuggestion, SimulationReport } from '@/types/workflows'

interface WorkflowState {
  conflicts: LogicConflict[]
  repairs: RepairSuggestion[]
  simulation: SimulationReport | null
  diagnostics: DiagnosticsSummary | null
}

export const useWorkflowStore = defineStore('workflows', {
  state: (): WorkflowState => ({
    conflicts: [],
    repairs: [],
    simulation: null,
    diagnostics: null,
  }),
  actions: {
    async runSampleAudit() {
      this.conflicts = await auditFacts([
        {
          id: 'fact_1',
          subject: '月光金属',
          predicate: 'state',
          object: 'solid',
          scopeTime: '第三纪',
          scopeLocation: '北方',
        },
        {
          id: 'fact_2',
          subject: '月光金属',
          predicate: 'state',
          object: 'liquid',
          scopeTime: '第三纪',
          scopeLocation: '北方',
        },
      ])
      if (this.conflicts[0]) {
        this.repairs = await repairSuggestions(this.conflicts[0])
      }
    },
    async runSampleSimulation(projectId: string) {
      this.simulation = await runSimulation(projectId, '如果北方发生饥荒', ['粮食', '北方城墙'])
    },
    async loadDiagnostics() {
      this.diagnostics = await diagnosticsSummary()
    },
  },
})
