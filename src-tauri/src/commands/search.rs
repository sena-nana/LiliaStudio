use tauri::{Manager, State};

use crate::{
    ai::{
        openai_compatible::{OpenAiCompatibleClient, UreqOpenAiTransport},
        settings::{api_key_secret_name, load_provider_settings, AiProviderKind, SecretStore},
    },
    db::app_state::AppState,
    services::{
        rag::{load_vector_candidates, rank_vector_candidates},
        search::{
            search_project, semantic_search_results, SearchFilter, SearchResult,
            SemanticSearchRequest, SemanticSearchResponse, SemanticSearchStatus,
        },
    },
    windows::credential::WindowsCredentialStore,
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

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible);
    let Some(provider) = provider else {
        return Ok(degraded("未找到 OpenAI 兼容接口设置"));
    };
    if !provider.enabled {
        return Ok(degraded("请先启用 OpenAI 兼容接口用于语义搜索"));
    }
    let Some(base_url) = provider.base_url.filter(|value| !value.trim().is_empty()) else {
        return Ok(degraded("请先填写 OpenAI 兼容接口的基础地址"));
    };
    let Some(model) = provider
        .embedding_model
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(degraded("请先填写 OpenAI 兼容接口的嵌入模型"));
    };
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty());
    let Some(api_key) = api_key else {
        return Ok(degraded("请先保存 OpenAI 兼容接口的接口密钥"));
    };

    let client = OpenAiCompatibleClient::new(base_url, api_key, UreqOpenAiTransport);
    let query_vector = match client.embeddings(&model, vec![request.query.clone()]) {
        Ok(result) => result.vectors.into_iter().next().unwrap_or_default(),
        Err(error) => {
            return Ok(SemanticSearchResponse {
                status: SemanticSearchStatus::Degraded,
                message: error.message,
                model,
                items: Vec::new(),
            });
        }
    };

    let candidates =
        state.with_database(|connection| load_vector_candidates(connection, &request.project_id))?;
    if candidates.is_empty() {
        return Ok(SemanticSearchResponse {
            status: SemanticSearchStatus::Degraded,
            message: "当前项目还没有可用的 embedding 索引。".into(),
            model,
            items: Vec::new(),
        });
    }
    let matches = rank_vector_candidates(candidates, query_vector, request.limit);
    let items =
        state.with_database(|connection| semantic_search_results(connection, matches, request.limit))?;

    Ok(SemanticSearchResponse {
        status: SemanticSearchStatus::Ready,
        message: String::new(),
        model,
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
