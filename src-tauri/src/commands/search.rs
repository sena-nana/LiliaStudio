use tauri::State;

use crate::{
    db::app_state::AppState,
    services::search::{search_project, SearchFilter, SearchResult},
};

#[tauri::command]
pub fn search_entities(
    state: State<'_, AppState>,
    filter: SearchFilter,
) -> Result<Vec<SearchResult>, String> {
    state.with_database(|connection| search_project(connection, filter))
}
