use tauri::State;

use crate::{
    db::app_state::AppState,
    services::import_export::{export_project, import_project, ImportedProject, ProjectArchive},
};

#[tauri::command]
pub fn export_project_archive(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectArchive, String> {
    state.with_database(|connection| export_project(connection, &project_id))
}

#[tauri::command]
pub fn import_project_archive(
    state: State<'_, AppState>,
    archive: ProjectArchive,
) -> Result<ImportedProject, String> {
    state.with_database(|connection| import_project(connection, archive))
}
