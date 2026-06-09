use tauri::State;

use crate::{
    db::app_state::AppState,
    domain::project::{Project, ProjectDraft, ProjectRepository},
};

#[tauri::command]
pub fn project_list(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    state.with_database(|connection| ProjectRepository::new(connection).list_active())
}

#[tauri::command]
pub fn project_create(state: State<'_, AppState>, draft: ProjectDraft) -> Result<Project, String> {
    state.with_database(|connection| ProjectRepository::new(connection).create(draft))
}

#[tauri::command]
pub fn project_update(
    state: State<'_, AppState>,
    id: String,
    draft: ProjectDraft,
) -> Result<Project, String> {
    state.with_database(|connection| ProjectRepository::new(connection).update(&id, draft))
}

#[tauri::command]
pub fn project_archive(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| ProjectRepository::new(connection).archive(&id))
}
