pub mod ai;
pub mod commands;
pub mod db;
pub mod domain;
pub mod logic;
pub mod services;
pub mod vector;
pub mod windows;
mod window_state;

pub mod test_support;

type WindowsRuntime = tauri_runtime_wry::Wry<tauri::EventLoopMessage>;

const MAIN_WINDOW_LABEL: &str = "main";
const BG: tauri::utils::config::Color = tauri::utils::config::Color(0x18, 0x18, 0x18, 0xFF);

pub fn run() {
    tauri::Builder::<WindowsRuntime>::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            use tauri::Manager;

            let app_data_dir = app.path().app_data_dir()?;
            let state = db::app_state::AppState::initialize(app_data_dir)?;
            app.manage(state);
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.set_background_color(Some(BG));
                if let Some(state) = window_state::load_main_window_state(app.handle()) {
                    window_state::restore_main_window_state(&window, state);
                }
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::{Manager, WindowEvent};

            if window.label() != MAIN_WINDOW_LABEL {
                return;
            }
            if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
                if let Some(webview_window) = window.get_webview_window(MAIN_WINDOW_LABEL) {
                    window_state::persist_main_window_state(&window.app_handle(), &webview_window);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::ai::default_ai_providers,
            commands::ai::load_ai_provider_settings,
            commands::ai::save_ai_provider_settings,
            commands::ai::test_openai_provider,
            commands::character_growth::preview_trait_delta,
            commands::cli_providers::test_claude_cli_provider,
            commands::cli_providers::test_codex_cli_provider,
            commands::diagnostics::diagnostics_summary,
            commands::health::health_check,
            commands::import_export::export_project_archive,
            commands::import_export::import_project_archive,
            commands::jobs::cancel_ai_job,
            commands::jobs::create_ai_job,
            commands::jobs::current_ai_job,
            commands::jobs::list_ai_job_logs,
            commands::jobs::list_ai_jobs,
            commands::jobs::retry_ai_job,
            commands::logic::audit_facts,
            commands::logic::repair_suggestions,
            commands::library::create_axiom,
            commands::library::create_character,
            commands::library::create_entry,
            commands::library::create_event,
            commands::library::create_relation,
            commands::library::delete_axiom,
            commands::library::delete_character,
            commands::library::delete_entry,
            commands::library::delete_event,
            commands::library::delete_relation,
            commands::library::list_backlinks,
            commands::library::list_characters,
            commands::library::list_entries,
            commands::library::list_events,
            commands::library::list_relations,
            commands::library::search_axioms,
            commands::library::update_axiom,
            commands::library::update_character,
            commands::library::update_entry,
            commands::library::update_event,
            commands::library::update_relation,
            commands::projects::archive_project,
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::prompts::copy_prompt_template,
            commands::prompts::list_prompt_templates,
            commands::prompts::preview_prompt_template,
            commands::prompts::reset_builtin_prompt_templates,
            commands::prompts::save_prompt_template,
            commands::rag::index_chunks,
            commands::rag::preview_context_pack,
            commands::search::search_entities,
            commands::simulation::run_simulation,
            commands::vector::preview_chunks
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Ameya");
}
