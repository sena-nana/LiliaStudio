use tauri::{Manager, State};

use crate::{
    ai::{
        openai_compatible::{OpenAiCompatibleClient, ProviderError, UreqOpenAiTransport},
        settings::{api_key_secret_name, load_provider_settings, AiProviderKind, SecretStore},
    },
    db::app_state::AppState,
    services::rag::{
        index_project_chunks, load_vector_candidates, rank_vector_candidates, upsert_embedding,
        ContextItem, ContextPack, DocumentChunkRecord,
    },
    windows::credential::WindowsCredentialStore,
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
pub fn rag_index_embeddings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, AppState>,
    project_id: String,
    max_chars: usize,
) -> Result<EmbeddingIndexResult, String> {
    let chunks = state
        .with_database(|connection| index_project_chunks(connection, &project_id, max_chars))?;
    if chunks.is_empty() {
        return Ok(EmbeddingIndexResult {
            chunk_count: 0,
            embedding_count: 0,
            model: String::new(),
        });
    }

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible && provider.enabled)
        .ok_or_else(|| "请先启用 OpenAI 兼容接口用于向量索引".to_string())?;
    let base_url = provider
        .base_url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先填写 OpenAI 兼容接口的基础地址".to_string())?;
    let model = provider
        .embedding_model
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先填写 OpenAI 兼容接口的嵌入模型".to_string())?;
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先保存 OpenAI 兼容接口的接口密钥".to_string())?;

    let client = OpenAiCompatibleClient::new(base_url, api_key, UreqOpenAiTransport);
    let result = client
        .embeddings(
            &model,
            chunks.iter().map(|chunk| chunk.text.clone()).collect(),
        )
        .map_err(provider_error_message)?;
    if result.vectors.len() != chunks.len() {
        return Err("嵌入模型返回的向量数量与切片数量不一致".into());
    }
    let embedding_count = result.vectors.len();
    state.with_database(|connection| {
        for (chunk, vector) in chunks.iter().zip(result.vectors.into_iter()) {
            upsert_embedding(connection, &chunk.id, &model, vector)?;
        }
        Ok(())
    })?;

    Ok(EmbeddingIndexResult {
        chunk_count: chunks.len(),
        embedding_count,
        model,
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
