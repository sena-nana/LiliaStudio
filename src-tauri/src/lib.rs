pub mod ai;
pub mod commands;
pub mod db;
pub mod domain;
pub mod logic;
pub mod services;
pub mod vector;

pub mod test_support;

type WindowsRuntime = tauri_runtime_wry::Wry<tauri::EventLoopMessage>;

pub fn run() {
    tauri::Builder::<WindowsRuntime>::default()
        .setup(|app| {
            use tauri::Manager;

            let app_data_dir = app.path().app_data_dir()?;
            let state = db::app_state::AppState::initialize(app_data_dir)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ai::default_ai_providers,
            commands::character_growth::preview_trait_delta,
            commands::diagnostics::diagnostics_summary,
            commands::health::health_check,
            commands::import_export::export_project_archive,
            commands::import_export::import_project_archive,
            commands::jobs::create_ai_job,
            commands::jobs::list_ai_jobs,
            commands::library::create_axiom,
            commands::library::create_character,
            commands::library::create_entry,
            commands::library::create_event,
            commands::library::create_relation,
            commands::library::list_backlinks,
            commands::library::list_characters,
            commands::library::list_entries,
            commands::library::list_events,
            commands::library::search_axioms,
            commands::projects::archive_project,
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::prompts::list_prompt_templates_command,
            commands::rag::index_chunks,
            commands::rag::preview_context_pack,
            commands::search::search_entities,
            commands::simulation::run_simulation,
            commands::vector::preview_chunks
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Ameya");
}
