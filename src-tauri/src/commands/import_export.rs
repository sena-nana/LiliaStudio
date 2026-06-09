use tauri::State;

use crate::{
    db::app_state::AppState,
    services::import_export::{export_project, import_project, ImportedProject, ProjectArchive},
};

#[tauri::command]
pub fn archive_export_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectArchive, String> {
    state.with_database(|connection| export_project(connection, &project_id))
}

#[tauri::command]
pub fn archive_import_project(
    state: State<'_, AppState>,
    archive: ProjectArchive,
) -> Result<ImportedProject, String> {
    state.with_database(|connection| import_project(connection, archive))
}
