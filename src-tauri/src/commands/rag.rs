use tauri::State;

use crate::{
    ai::{
        openai_compatible::{OpenAiCompatibleClient, ProviderError, UreqOpenAiTransport},
        provider_resolver::{
            load_openai_settings, resolve_openai_embedding_provider,
            resolve_optional_openai_embedding_model, OPENAI_RAG_MESSAGES,
        },
    },
    db::app_state::AppState,
    services::rag::{
        chunks_requiring_embedding, embedding_index_status, index_project_chunks,
        load_vector_candidates, rank_vector_candidates, upsert_embedding, ContextItem, ContextPack,
        DocumentChunkRecord, RagIndexStatus,
    },
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingIndexResult {
    pub chunk_count: usize,
    pub embedding_count: usize,
    pub model: String,
}

#[tauri::command]
pub fn rag_index_chunks(
    state: State<'_, AppState>,
    project_id: String,
    max_chars: usize,
) -> Result<Vec<DocumentChunkRecord>, String> {
    state.with_database(|connection| index_project_chunks(connection, &project_id, max_chars))
}

#[tauri::command]
pub fn rag_index_status<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, AppState>,
    project_id: String,
) -> Result<RagIndexStatus, String> {
    let (settings, api_key) = load_openai_settings(&app)?;
    let model = resolve_optional_openai_embedding_model(settings, api_key);
    state.with_database(|connection| {
        embedding_index_status(connection, &project_id, model.as_deref())
    })
}

#[tauri::command]
pub fn rag_index_embeddings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, AppState>,
    project_id: String,
    max_chars: usize,
) -> Result<EmbeddingIndexResult, String> {
    state.with_database(|connection| index_project_chunks(connection, &project_id, max_chars))?;
    let (settings, api_key) = load_openai_settings(&app)?;
    let provider = resolve_openai_embedding_provider(settings, api_key, OPENAI_RAG_MESSAGES)?;
    let chunks = state.with_database(|connection| {
        chunks_requiring_embedding(connection, &project_id, &provider.model)
    })?;
    if chunks.is_empty() {
        let status = state.with_database(|connection| {
            embedding_index_status(connection, &project_id, Some(&provider.model))
        })?;
        return Ok(EmbeddingIndexResult {
            chunk_count: status.chunk_count,
            embedding_count: status.embedding_count,
            model: status.model,
        });
    }

    let client =
        OpenAiCompatibleClient::new(provider.base_url, provider.api_key, UreqOpenAiTransport);
    let result = client
        .embeddings(
            &provider.model,
            chunks.iter().map(|chunk| chunk.text.clone()).collect(),
        )
        .map_err(provider_error_message)?;
    if result.vectors.len() != chunks.len() {
        return Err("嵌入模型返回的向量数量与切片数量不一致".into());
    }
    state.with_database(|connection| {
        for (chunk, vector) in chunks.iter().zip(result.vectors.into_iter()) {
            upsert_embedding(connection, &chunk.id, &provider.model, vector)?;
        }
        Ok(())
    })?;
    let status = state.with_database(|connection| {
        embedding_index_status(connection, &project_id, Some(&provider.model))
    })?;

    Ok(EmbeddingIndexResult {
        chunk_count: status.chunk_count,
        embedding_count: status.embedding_count,
        model: status.model,
    })
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

fn provider_error_message(error: ProviderError) -> String {
    error.message
}
