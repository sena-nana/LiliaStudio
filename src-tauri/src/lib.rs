pub mod commands;
pub mod db;
pub mod domain;

pub mod test_support;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;

            let app_data_dir = app.path().app_data_dir()?;
            let state = db::app_state::AppState::initialize(app_data_dir)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
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
            commands::projects::update_project
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Ameya");
}
