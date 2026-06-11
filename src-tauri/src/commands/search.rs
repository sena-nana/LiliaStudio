use tauri::State;

use crate::{
    ai::{
        openai_compatible::{OpenAiCompatibleClient, UreqOpenAiTransport},
        provider_resolver::{
            load_openai_settings, resolve_openai_embedding_provider, OPENAI_SEARCH_MESSAGES,
        },
    },
    db::app_state::AppState,
    services::{
        rag::{embedding_index_status, load_vector_candidates_for_model, rank_vector_candidates},
        search::{
            search_project, semantic_search_results, SearchFilter, SearchResult,
            SemanticSearchRequest, SemanticSearchResponse, SemanticSearchStatus,
        },
    },
};

#[tauri::command]
pub fn search_entities(
    state: State<'_, AppState>,
    filter: SearchFilter,
) -> Result<Vec<SearchResult>, String> {
    state.with_database(|connection| search_project(connection, filter))
}

#[tauri::command]
pub fn search_semantic<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, AppState>,
    request: SemanticSearchRequest,
) -> Result<SemanticSearchResponse, String> {
    if request.query.trim().is_empty() {
        return Ok(SemanticSearchResponse {
            status: SemanticSearchStatus::Ready,
            message: String::new(),
            model: String::new(),
            items: Vec::new(),
        });
    }

    let (settings, api_key) = load_openai_settings(&app)?;
    let provider =
        match resolve_openai_embedding_provider(settings, api_key, OPENAI_SEARCH_MESSAGES) {
            Ok(provider) => provider,
            Err(message) => return Ok(degraded(message)),
        };

    let index_status = state.with_database(|connection| {
        embedding_index_status(connection, &request.project_id, Some(&provider.model))
    })?;
    if index_status.status != "ready" {
        return Ok(SemanticSearchResponse {
            status: SemanticSearchStatus::Degraded,
            message: index_status.message,
            model: provider.model,
            items: Vec::new(),
        });
    }

    let client =
        OpenAiCompatibleClient::new(provider.base_url, provider.api_key, UreqOpenAiTransport);
    let query_vector = match client.embeddings(&provider.model, vec![request.query.clone()]) {
        Ok(result) => result.vectors.into_iter().next().unwrap_or_default(),
        Err(error) => {
            return Ok(SemanticSearchResponse {
                status: SemanticSearchStatus::Degraded,
                message: error.message,
                model: provider.model,
                items: Vec::new(),
            });
        }
    };

    let candidates = state.with_database(|connection| {
        load_vector_candidates_for_model(connection, &request.project_id, &provider.model)
    })?;
    if candidates.is_empty() {
        return Ok(SemanticSearchResponse {
            status: SemanticSearchStatus::Degraded,
            message: "当前项目还没有可用的 embedding 索引。".into(),
            model: provider.model,
            items: Vec::new(),
        });
    }
    let matches = rank_vector_candidates(candidates, query_vector, request.limit);
    let items = state
        .with_database(|connection| semantic_search_results(connection, matches, request.limit))?;

    Ok(SemanticSearchResponse {
        status: SemanticSearchStatus::Ready,
        message: String::new(),
        model: provider.model,
        items,
    })
}

fn degraded(message: impl Into<String>) -> SemanticSearchResponse {
    SemanticSearchResponse {
        status: SemanticSearchStatus::Degraded,
        message: message.into(),
        model: String::new(),
        items: Vec::new(),
    }
}
