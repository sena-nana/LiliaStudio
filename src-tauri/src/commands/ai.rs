use tauri::Manager;

use crate::{
    ai::{
        openai_compatible::{
            ChatMessage, OpenAiCompatibleClient, ProviderError, UreqOpenAiTransport,
        },
        settings::{
            api_key_secret_name, load_provider_settings, save_provider_settings, AiProviderConfig,
            AiProviderKind, AiProviderSettingsDraft, AiProviderSettingsView, SecretStore,
        },
    },
    windows::credential::WindowsCredentialStore,
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiProviderTestResult {
    pub ok: bool,
    pub message: String,
    pub error: Option<ProviderError>,
}

#[tauri::command]
pub fn default_ai_providers() -> Vec<AiProviderConfig> {
    vec![
        AiProviderConfig::codex_cli(),
        AiProviderConfig::claude_cli(),
    ]
}

#[tauri::command]
pub fn load_ai_provider_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<AiProviderSettingsView>, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    load_provider_settings(&config_dir, &WindowsCredentialStore)
}

#[tauri::command]
pub fn save_ai_provider_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    drafts: Vec<AiProviderSettingsDraft>,
) -> Result<Vec<AiProviderSettingsView>, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    save_provider_settings(&config_dir, drafts, &WindowsCredentialStore)
}

#[tauri::command]
pub fn test_openai_provider<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<OpenAiProviderTestResult, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible)
        .ok_or_else(|| "未找到 OpenAI-compatible Provider 设置".to_string())?;
    let Some(base_url) = provider.base_url.filter(|value| !value.trim().is_empty()) else {
        return Ok(error_result(ProviderError::config_missing(
            "请先填写 OpenAI-compatible Base URL",
        )));
    };
    let Some(model) = provider.chat_model.filter(|value| !value.trim().is_empty()) else {
        return Ok(error_result(ProviderError::config_missing(
            "请先填写 OpenAI-compatible Chat model",
        )));
    };
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty());
    let Some(api_key) = api_key else {
        return Ok(error_result(ProviderError::config_missing(
            "请先保存 OpenAI-compatible API Key",
        )));
    };

    let client = OpenAiCompatibleClient::new(base_url, api_key, UreqOpenAiTransport);
    match client.chat(
        &model,
        vec![ChatMessage {
            role: "user".into(),
            content: "Return the word ok.".into(),
        }],
        0.0,
    ) {
        Ok(content) => Ok(OpenAiProviderTestResult {
            ok: true,
            message: if content.trim().is_empty() {
                "Provider 已响应，但返回内容为空".into()
            } else {
                "Provider 测试调用成功".into()
            },
            error: None,
        }),
        Err(error) => Ok(error_result(error)),
    }
}

fn error_result(error: ProviderError) -> OpenAiProviderTestResult {
    OpenAiProviderTestResult {
        ok: false,
        message: error.message.clone(),
        error: Some(error),
    }
}
