pub mod ai;
pub mod character_growth;
pub mod cli_providers;
pub mod diagnostics;
pub mod health;
pub mod import_export;
pub mod jobs;
pub mod library;
pub mod logic;
pub mod projects;
pub mod prompts;
pub mod rag;
pub mod search;
pub mod simulation;
pub mod vector;

pub fn register_handlers(
    builder: tauri::Builder<crate::WindowsRuntime>,
) -> tauri::Builder<crate::WindowsRuntime> {
    builder.invoke_handler(tauri::generate_handler![
        ai::ai_default_providers,
        ai::ai_agent_ask,
        ai::ai_load_provider_settings,
        ai::ai_save_provider_settings,
        ai::ai_test_openai_provider,
        character_growth::character_growth_create_record,
        character_growth::character_growth_preview_trait_delta,
        character_growth::character_growth_workspace,
        cli_providers::ai_test_claude_cli_provider,
        cli_providers::ai_test_codex_cli_provider,
        diagnostics::diagnostics_summary,
        health::health_check,
        import_export::archive_export_project,
        import_export::archive_import_project,
        jobs::job_cancel,
        jobs::job_create,
        jobs::job_current,
        jobs::job_logs,
        jobs::job_list,
        jobs::job_retry,
        logic::logic_audit_facts,
        logic::logic_repair_suggestions,
        library::library_create_axiom,
        library::library_create_character,
        library::library_create_entry,
        library::library_create_event,
        library::library_create_relation,
        library::library_delete_axiom,
        library::library_delete_character,
        library::library_delete_entry,
        library::library_delete_event,
        library::library_delete_relation,
        library::library_list_backlinks,
        library::library_project_snapshot,
        library::library_relation_neighborhood,
        library::library_relation_type_presets,
        library::library_update_axiom,
        library::library_update_character,
        library::library_update_entry,
        library::library_update_event,
        library::library_update_relation,
        projects::project_archive,
        projects::project_create,
        projects::project_list,
        projects::project_update,
        prompts::prompt_copy_template,
        prompts::prompt_list_templates,
        prompts::prompt_preview_template,
        prompts::prompt_reset_builtin_templates,
        prompts::prompt_save_template,
        rag::rag_index_chunks,
        rag::rag_index_embeddings,
        rag::rag_preview_context_pack,
        search::search_entities,
        search::search_semantic,
        simulation::simulation_run,
        vector::vector_preview_chunks
    ])
}
