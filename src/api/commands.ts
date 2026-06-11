import type {
  AiJob,
  AiJobDraft,
  AiJobLog,
  AgentAskRequest,
  AgentAskResponse,
  AiProviderConfig,
  AiProviderSettingsDraft,
  AiProviderSettingsView,
  CliProviderTestResult,
  ContextPack,
  DocumentChunkRecord,
  OpenAiProviderTestResult,
  PromptTemplate,
  PromptTemplateDraft,
  PromptTemplatePreview,
  PromptTemplateVariableValue,
  RagIndexStatus,
  TextChunk,
} from "@/types/ai";
import type { ImportedProject, ProjectArchive } from "@/types/archive";
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
  RelationNeighborhood,
} from "@/types/library";
import type { Project, ProjectDraft } from "@/types/project";
import type {
  SearchFilter,
  SearchResult,
  SemanticSearchRequest,
  SemanticSearchResponse,
} from "@/types/search";
import type {
  CharacterTraitState,
  CharacterGrowthWorkspaceSnapshot,
  CharacterTraitDeltaRecord,
  CharacterTraitDeltaRecordDraft,
  DiagnosticsSummary,
  Fact,
  LogicConflict,
  RepairSuggestion,
  SimulationReport,
  TraitDelta,
} from "@/types/workflows";
import type { HealthInfo } from "./health";

export interface CommandDefinition<Args, Result> {
  name: string;
  __args?: Args;
  __result?: Result;
}

export type CommandArgs<Command> =
  Command extends CommandDefinition<infer Args, unknown> ? Args : never;

export type CommandResult<Command> =
  Command extends CommandDefinition<unknown, infer Result> ? Result : never;

function defineCommand<Args, Result>(name: string): CommandDefinition<Args, Result> {
  return { name };
}

export const commands = {
  ai: {
    agentAsk: defineCommand<{ request: AgentAskRequest }, AgentAskResponse>("ai_agent_ask"),
    defaultProviders: defineCommand<undefined, AiProviderConfig[]>("ai_default_providers"),
    loadProviderSettings: defineCommand<undefined, AiProviderSettingsView[]>("ai_load_provider_settings"),
    saveProviderSettings: defineCommand<{ drafts: AiProviderSettingsDraft[] }, AiProviderSettingsView[]>(
      "ai_save_provider_settings",
    ),
    testOpenAiProvider: defineCommand<undefined, OpenAiProviderTestResult>("ai_test_openai_provider"),
    testCodexCliProvider: defineCommand<undefined, CliProviderTestResult>("ai_test_codex_cli_provider"),
    testClaudeCliProvider: defineCommand<undefined, CliProviderTestResult>("ai_test_claude_cli_provider"),
  },
  archive: {
    exportProject: defineCommand<{ projectId: string }, ProjectArchive>("archive_export_project"),
    importProject: defineCommand<{ archive: ProjectArchive }, ImportedProject>("archive_import_project"),
  },
  diagnostics: {
    summary: defineCommand<undefined, DiagnosticsSummary>("diagnostics_summary"),
  },
  health: {
    check: defineCommand<undefined, HealthInfo>("health_check"),
  },
  job: {
    create: defineCommand<{ draft: AiJobDraft }, AiJob>("job_create"),
    list: defineCommand<undefined, AiJob[]>("job_list"),
    current: defineCommand<undefined, AiJob | null>("job_current"),
    logs: defineCommand<{ jobId: string }, AiJobLog[]>("job_logs"),
    cancel: defineCommand<{ jobId: string }, AiJob>("job_cancel"),
    retry: defineCommand<{ jobId: string }, AiJob>("job_retry"),
  },
  library: {
    projectSnapshot: defineCommand<{ projectId: string }, LibraryProjectSnapshot>("library_project_snapshot"),
    createEntry: defineCommand<{ draft: EntryDraft }, Entry>("library_create_entry"),
    updateEntry: defineCommand<{ id: string; draft: EntryDraft }, Entry>("library_update_entry"),
    deleteEntry: defineCommand<{ id: string }, void>("library_delete_entry"),
    createCharacter: defineCommand<{ draft: CharacterDraft }, Character>("library_create_character"),
    updateCharacter: defineCommand<{ id: string; draft: CharacterDraft }, Character>("library_update_character"),
    deleteCharacter: defineCommand<{ id: string }, void>("library_delete_character"),
    createEvent: defineCommand<
      { draft: EventDraft; participants: EventParticipantDraft[] },
      EventRecord
    >("library_create_event"),
    updateEvent: defineCommand<
      { id: string; draft: EventDraft; participants: EventParticipantDraft[] },
      EventRecord
    >("library_update_event"),
    deleteEvent: defineCommand<{ id: string }, void>("library_delete_event"),
    createAxiom: defineCommand<{ draft: AxiomDraft }, Axiom>("library_create_axiom"),
    updateAxiom: defineCommand<{ id: string; draft: AxiomDraft }, Axiom>("library_update_axiom"),
    deleteAxiom: defineCommand<{ id: string }, void>("library_delete_axiom"),
    listBacklinks: defineCommand<{ target: EntityRef }, Relation[]>("library_list_backlinks"),
    relationTypePresets: defineCommand<undefined, string[]>("library_relation_type_presets"),
    relationNeighborhood: defineCommand<
      { projectId: string; target: EntityRef; depth: number },
      RelationNeighborhood
    >("library_relation_neighborhood"),
    createRelation: defineCommand<{ draft: RelationDraft }, Relation>("library_create_relation"),
    updateRelation: defineCommand<{ id: string; draft: RelationDraft }, Relation>("library_update_relation"),
    deleteRelation: defineCommand<{ id: string }, void>("library_delete_relation"),
  },
  logic: {
    auditFacts: defineCommand<{ facts: Fact[] }, LogicConflict[]>("logic_audit_facts"),
    repairSuggestions: defineCommand<{ conflict: LogicConflict }, RepairSuggestion[]>("logic_repair_suggestions"),
  },
  project: {
    list: defineCommand<undefined, Project[]>("project_list"),
    create: defineCommand<{ draft: ProjectDraft }, Project>("project_create"),
    update: defineCommand<{ id: string; draft: ProjectDraft }, Project>("project_update"),
    archive: defineCommand<{ id: string }, void>("project_archive"),
  },
  prompt: {
    listTemplates: defineCommand<undefined, PromptTemplate[]>("prompt_list_templates"),
    copyTemplate: defineCommand<{ templateId: string }, PromptTemplate>("prompt_copy_template"),
    saveTemplate: defineCommand<{ draft: PromptTemplateDraft }, PromptTemplate>("prompt_save_template"),
    resetBuiltinTemplates: defineCommand<undefined, PromptTemplate[]>("prompt_reset_builtin_templates"),
    previewTemplate: defineCommand<
      { request: { template: string; values: PromptTemplateVariableValue[] } },
      PromptTemplatePreview
    >("prompt_preview_template"),
  },
  rag: {
    indexChunks: defineCommand<{ projectId: string; maxChars: number }, DocumentChunkRecord[]>("rag_index_chunks"),
    indexStatus: defineCommand<{ projectId: string }, RagIndexStatus>("rag_index_status"),
    indexEmbeddings: defineCommand<
      { projectId: string; maxChars: number },
      { chunkCount: number; embeddingCount: number; model: string }
    >("rag_index_embeddings"),
    previewContextPack: defineCommand<
      { projectId: string; query: string; queryVector: number[] },
      ContextPack
    >("rag_preview_context_pack"),
  },
  search: {
    entities: defineCommand<{ filter: SearchFilter }, SearchResult[]>("search_entities"),
    semantic: defineCommand<{ request: SemanticSearchRequest }, SemanticSearchResponse>(
      "search_semantic",
    ),
  },
  simulation: {
    run: defineCommand<
      { projectId: string; scenario: string; referencedEntities: string[] },
      SimulationReport
    >("simulation_run"),
  },
  vector: {
    previewChunks: defineCommand<{ text: string; maxChars: number }, TextChunk[]>("vector_preview_chunks"),
  },
  characterGrowth: {
    previewTraitDelta: defineCommand<
      { state: CharacterTraitState; delta: TraitDelta },
      CharacterTraitState
    >("character_growth_preview_trait_delta"),
    workspace: defineCommand<{ projectId: string }, CharacterGrowthWorkspaceSnapshot>(
      "character_growth_workspace",
    ),
    createRecord: defineCommand<{ draft: CharacterTraitDeltaRecordDraft }, CharacterTraitDeltaRecord>(
      "character_growth_create_record",
    ),
  },
} as const;
