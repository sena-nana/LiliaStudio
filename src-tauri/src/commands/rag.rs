use tauri::State;

use crate::{
    db::app_state::AppState,
    services::rag::{build_context_pack, index_project_chunks, ContextPack, DocumentChunkRecord},
};

#[tauri::command]
pub fn index_chunks(
    state: State<'_, AppState>,
    project_id: String,
    max_chars: usize,
) -> Result<Vec<DocumentChunkRecord>, String> {
    state.with_database(|connection| index_project_chunks(connection, &project_id, max_chars))
}

#[tauri::command]
pub fn preview_context_pack(
    state: State<'_, AppState>,
    project_id: String,
    query: String,
    query_vector: Vec<f32>,
) -> Result<ContextPack, String> {
    state.with_database(|connection| build_context_pack(connection, &project_id, &query, query_vector))
}
