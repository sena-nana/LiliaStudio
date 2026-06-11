use tauri::Manager;

use crate::{
    ai::settings::{
        api_key_secret_name, load_provider_settings, AiProviderKind, AiProviderSettingsView,
        SecretStore,
    },
    windows::credential::WindowsCredentialStore,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiChatProvider {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiEmbeddingProvider {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Copy)]
pub struct OpenAiResolveMessages {
    pub missing_provider: &'static str,
    pub disabled: Option<&'static str>,
    pub missing_base_url: &'static str,
    pub missing_chat_model: &'static str,
    pub missing_embedding_model: &'static str,
    pub missing_api_key: &'static str,
}

pub const OPENAI_TEST_MESSAGES: OpenAiResolveMessages = OpenAiResolveMessages {
    missing_provider: "未找到 OpenAI 兼容接口设置",
    disabled: None,
    missing_base_url: "请先填写 OpenAI 兼容接口的基础地址",
    missing_chat_model: "请先填写 OpenAI 兼容接口的对话模型",
    missing_embedding_model: "请先填写 OpenAI 兼容接口的嵌入模型",
    missing_api_key: "请先保存 OpenAI 兼容接口的接口密钥",
};

pub const OPENAI_AGENT_MESSAGES: OpenAiResolveMessages = OpenAiResolveMessages {
    missing_provider: "请先在 AI 设置中启用 OpenAI 兼容接口",
    disabled: Some("请先在 AI 设置中启用 OpenAI 兼容接口"),
    missing_base_url: "请先填写 OpenAI 兼容接口的基础地址",
    missing_chat_model: "请先填写 OpenAI 兼容接口的对话模型",
    missing_embedding_model: "请先填写 OpenAI 兼容接口的嵌入模型",
    missing_api_key: "请先保存 OpenAI 兼容接口的接口密钥",
};

pub const OPENAI_RAG_MESSAGES: OpenAiResolveMessages = OpenAiResolveMessages {
    missing_provider: "请先启用 OpenAI 兼容接口用于向量索引",
    disabled: Some("请先启用 OpenAI 兼容接口用于向量索引"),
    missing_base_url: "请先填写 OpenAI 兼容接口的基础地址",
    missing_chat_model: "请先填写 OpenAI 兼容接口的对话模型",
    missing_embedding_model: "请先填写 OpenAI 兼容接口的嵌入模型",
    missing_api_key: "请先保存 OpenAI 兼容接口的接口密钥",
};

pub const OPENAI_SEARCH_MESSAGES: OpenAiResolveMessages = OpenAiResolveMessages {
    missing_provider: "未找到 OpenAI 兼容接口设置",
    disabled: Some("请先启用 OpenAI 兼容接口用于语义搜索"),
    missing_base_url: "请先填写 OpenAI 兼容接口的基础地址",
    missing_chat_model: "请先填写 OpenAI 兼容接口的对话模型",
    missing_embedding_model: "请先填写 OpenAI 兼容接口的嵌入模型",
    missing_api_key: "请先保存 OpenAI 兼容接口的接口密钥",
};

pub fn load_openai_settings<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(Vec<AiProviderSettingsView>, Option<String>), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty());
    Ok((settings, api_key))
}

pub fn resolve_openai_chat_provider(
    settings: Vec<AiProviderSettingsView>,
    api_key: Option<String>,
    messages: OpenAiResolveMessages,
) -> Result<OpenAiChatProvider, String> {
    let provider = openai_provider(settings, messages)?;
    let base_url = required_value(provider.base_url, messages.missing_base_url)?;
    let model = required_value(provider.chat_model, messages.missing_chat_model)?;
    let api_key = required_value(api_key, messages.missing_api_key)?;

    Ok(OpenAiChatProvider {
        base_url,
        model,
        api_key,
    })
}

pub fn resolve_openai_embedding_provider(
    settings: Vec<AiProviderSettingsView>,
    api_key: Option<String>,
    messages: OpenAiResolveMessages,
) -> Result<OpenAiEmbeddingProvider, String> {
    let provider = openai_provider(settings, messages)?;
    let base_url = required_value(provider.base_url, messages.missing_base_url)?;
    let model = required_value(provider.embedding_model, messages.missing_embedding_model)?;
    let api_key = required_value(api_key, messages.missing_api_key)?;

    Ok(OpenAiEmbeddingProvider {
        base_url,
        model,
        api_key,
    })
}

pub fn resolve_optional_openai_embedding_model(
    settings: Vec<AiProviderSettingsView>,
    api_key: Option<String>,
) -> Option<String> {
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible && provider.enabled)?;
    required_value(provider.base_url, "").ok()?;
    required_value(api_key, "").ok()?;
    required_value(provider.embedding_model, "").ok()
}

fn openai_provider(
    settings: Vec<AiProviderSettingsView>,
    messages: OpenAiResolveMessages,
) -> Result<AiProviderSettingsView, String> {
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible)
        .ok_or_else(|| messages.missing_provider.to_string())?;
    if let Some(message) = messages.disabled {
        if !provider.enabled {
            return Err(message.to_string());
        }
    }
    Ok(provider)
}

fn required_value(value: Option<String>, message: &str) -> Result<String, String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message.to_string())
}

#[cfg(test)]
mod tests {
    use crate::ai::settings::{AiProviderKind, AiProviderSettingsView};

    use super::{
        resolve_openai_chat_provider, resolve_openai_embedding_provider,
        resolve_optional_openai_embedding_model, OPENAI_AGENT_MESSAGES, OPENAI_RAG_MESSAGES,
        OPENAI_SEARCH_MESSAGES,
    };

    fn openai_settings(
        overrides: impl FnOnce(&mut AiProviderSettingsView),
    ) -> AiProviderSettingsView {
        let mut settings = AiProviderSettingsView {
            kind: AiProviderKind::OpenAiCompatible,
            base_url: Some("https://llm.example/v1".into()),
            api_key_preview: None,
            has_api_key: true,
            chat_model: Some("story-chat".into()),
            embedding_model: Some("story-embed".into()),
            command_template: None,
            enabled: true,
        };
        overrides(&mut settings);
        settings
    }

    #[test]
    fn chat_provider_requires_enabled_openai_provider() {
        let error = resolve_openai_chat_provider(
            vec![openai_settings(|settings| settings.enabled = false)],
            Some("sk-test".into()),
            OPENAI_AGENT_MESSAGES,
        )
        .expect_err("disabled provider should fail");

        assert_eq!(error, "请先在 AI 设置中启用 OpenAI 兼容接口");
    }

    #[test]
    fn embedding_provider_uses_contextual_disabled_message() {
        let error = resolve_openai_embedding_provider(
            vec![openai_settings(|settings| settings.enabled = false)],
            Some("sk-test".into()),
            OPENAI_RAG_MESSAGES,
        )
        .expect_err("disabled provider should fail");

        assert_eq!(error, "请先启用 OpenAI 兼容接口用于向量索引");
    }

    #[test]
    fn embedding_provider_can_report_search_degraded_message() {
        let error = resolve_openai_embedding_provider(
            vec![openai_settings(|settings| settings.enabled = false)],
            Some("sk-test".into()),
            OPENAI_SEARCH_MESSAGES,
        )
        .expect_err("disabled provider should fail");

        assert_eq!(error, "请先启用 OpenAI 兼容接口用于语义搜索");
    }

    #[test]
    fn provider_requires_base_url_model_and_api_key() {
        let missing_base = resolve_openai_chat_provider(
            vec![openai_settings(|settings| {
                settings.base_url = Some("  ".into())
            })],
            Some("sk-test".into()),
            OPENAI_AGENT_MESSAGES,
        )
        .expect_err("base url is required");
        let missing_chat_model = resolve_openai_chat_provider(
            vec![openai_settings(|settings| settings.chat_model = None)],
            Some("sk-test".into()),
            OPENAI_AGENT_MESSAGES,
        )
        .expect_err("chat model is required");
        let missing_embedding_model = resolve_openai_embedding_provider(
            vec![openai_settings(|settings| settings.embedding_model = None)],
            Some("sk-test".into()),
            OPENAI_RAG_MESSAGES,
        )
        .expect_err("embedding model is required");
        let missing_api_key = resolve_openai_chat_provider(
            vec![openai_settings(|_| {})],
            None,
            OPENAI_AGENT_MESSAGES,
        )
        .expect_err("api key is required");

        assert_eq!(missing_base, "请先填写 OpenAI 兼容接口的基础地址");
        assert_eq!(missing_chat_model, "请先填写 OpenAI 兼容接口的对话模型");
        assert_eq!(
            missing_embedding_model,
            "请先填写 OpenAI 兼容接口的嵌入模型"
        );
        assert_eq!(missing_api_key, "请先保存 OpenAI 兼容接口的接口密钥");
    }

    #[test]
    fn provider_returns_successful_chat_and_embedding_configs() {
        let chat = resolve_openai_chat_provider(
            vec![openai_settings(|_| {})],
            Some("sk-test".into()),
            OPENAI_AGENT_MESSAGES,
        )
        .expect("chat provider resolves");
        let embedding = resolve_openai_embedding_provider(
            vec![openai_settings(|_| {})],
            Some("sk-test".into()),
            OPENAI_RAG_MESSAGES,
        )
        .expect("embedding provider resolves");

        assert_eq!(chat.base_url, "https://llm.example/v1");
        assert_eq!(chat.model, "story-chat");
        assert_eq!(chat.api_key, "sk-test");
        assert_eq!(embedding.model, "story-embed");
    }

    #[test]
    fn optional_embedding_model_requires_ready_embedding_provider() {
        assert_eq!(
            resolve_optional_openai_embedding_model(
                vec![openai_settings(
                    |settings| settings.embedding_model = Some("story-embed".into())
                )],
                Some("sk-test".into()),
            ),
            Some("story-embed".into()),
        );
        assert_eq!(
            resolve_optional_openai_embedding_model(
                vec![openai_settings(|settings| settings.base_url = None)],
                Some("sk-test".into()),
            ),
            None,
        );
    }
}
