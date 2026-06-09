use tauri::State;

use crate::{
    db::app_state::AppState,
    services::rag::{
        index_project_chunks, load_vector_candidates, rank_vector_candidates, ContextItem,
        ContextPack, DocumentChunkRecord,
    },
};

#[tauri::command]
pub fn rag_index_chunks(
    state: State<'_, AppState>,
    project_id: String,
    max_chars: usize,
) -> Result<Vec<DocumentChunkRecord>, String> {
    state.with_database(|connection| index_project_chunks(connection, &project_id, max_chars))
}

#[tauri::command]
pub fn rag_preview_context_pack(
    state: State<'_, AppState>,
    project_id: String,
    query: String,
    query_vector: Vec<f32>,
) -> Result<ContextPack, String> {
    let candidates =
        state.with_database(|connection| load_vector_candidates(connection, &project_id))?;
    let items = rank_vector_candidates(candidates, query_vector, 8)
        .into_iter()
        .map(|item| ContextItem {
            source_type: item.source_type,
            source_id: item.source_id,
            text: item.text,
            score: item.score,
        })
        .collect();
    Ok(ContextPack {
        project_id,
        query,
        items,
    })
}
