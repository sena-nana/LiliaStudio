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

const AGENT_SYSTEM_PROMPT: &str = "你是 Ameya 的项目资料问答助手。只能基于用户提供的项目上下文回答；如果上下文不足以支持结论，明确说明不确定，并指出还需要哪些资料。回答要简洁、可执行，并在相关结论后引用上下文编号。";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiProviderTestResult {
    pub ok: bool,
    pub message: String,
    pub error: Option<ProviderError>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskReference {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub source: String,
    pub score: f32,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskRequest {
    pub project_id: String,
    pub question: String,
    pub references: Vec<AgentAskReference>,
}

#[derive(Debug, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskResponse {
    pub answer: String,
    pub references: Vec<AgentAskReference>,
    pub provider_kind: AiProviderKind,
    pub model: String,
}

#[derive(Debug)]
struct OpenAiAgentProvider {
    base_url: String,
    model: String,
    api_key: String,
}

#[tauri::command]
pub fn ai_default_providers() -> Vec<AiProviderConfig> {
    vec![
        AiProviderConfig::codex_cli(),
        AiProviderConfig::claude_cli(),
    ]
}

#[tauri::command]
pub fn ai_load_provider_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<AiProviderSettingsView>, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    load_provider_settings(&config_dir, &WindowsCredentialStore)
}

#[tauri::command]
pub fn ai_save_provider_settings<R: tauri::Runtime>(
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
pub fn ai_test_openai_provider<R: tauri::Runtime>(
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
        .ok_or_else(|| "未找到 OpenAI 兼容接口设置".to_string())?;
    let Some(base_url) = provider.base_url.filter(|value| !value.trim().is_empty()) else {
        return Ok(error_result(ProviderError::config_missing(
            "请先填写 OpenAI 兼容接口的基础地址",
        )));
    };
    let Some(model) = provider.chat_model.filter(|value| !value.trim().is_empty()) else {
        return Ok(error_result(ProviderError::config_missing(
            "请先填写 OpenAI 兼容接口的对话模型",
        )));
    };
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty());
    let Some(api_key) = api_key else {
        return Ok(error_result(ProviderError::config_missing(
            "请先保存 OpenAI 兼容接口的接口密钥",
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

#[tauri::command]
pub fn ai_agent_ask<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    request: AgentAskRequest,
) -> Result<AgentAskResponse, String> {
    let question = request.question.trim();
    if question.is_empty() {
        return Err("请输入问题".into());
    }

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let api_key = WindowsCredentialStore
        .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))?
        .filter(|value| !value.trim().is_empty());
    let provider = resolve_openai_agent_provider(settings, api_key)?;

    let references = request.references;
    let messages = build_agent_messages(&request.project_id, question, &references);
    let client =
        OpenAiCompatibleClient::new(provider.base_url, provider.api_key, UreqOpenAiTransport);
    let answer = client
        .chat(&provider.model, messages, 0.2)
        .map_err(|error| error.message)?;

    Ok(AgentAskResponse {
        answer,
        references,
        provider_kind: AiProviderKind::OpenAiCompatible,
        model: provider.model,
    })
}

fn error_result(error: ProviderError) -> OpenAiProviderTestResult {
    OpenAiProviderTestResult {
        ok: false,
        message: error.message.clone(),
        error: Some(error),
    }
}

fn resolve_openai_agent_provider(
    settings: Vec<AiProviderSettingsView>,
    api_key: Option<String>,
) -> Result<OpenAiAgentProvider, String> {
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::OpenAiCompatible && provider.enabled)
        .ok_or_else(|| "请先在 AI 设置中启用 OpenAI 兼容接口".to_string())?;
    let base_url = provider
        .base_url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先填写 OpenAI 兼容接口的基础地址".to_string())?;
    let model = provider
        .chat_model
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先填写 OpenAI 兼容接口的对话模型".to_string())?;
    let api_key = api_key
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "请先保存 OpenAI 兼容接口的接口密钥".to_string())?;

    Ok(OpenAiAgentProvider {
        base_url,
        model,
        api_key,
    })
}

fn build_agent_messages(
    project_id: &str,
    question: &str,
    references: &[AgentAskReference],
) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "system".into(),
            content: AGENT_SYSTEM_PROMPT.into(),
        },
        ChatMessage {
            role: "user".into(),
            content: build_agent_user_prompt(project_id, question, references),
        },
    ]
}

fn build_agent_user_prompt(
    project_id: &str,
    question: &str,
    references: &[AgentAskReference],
) -> String {
    let mut prompt = format!("项目 ID：{project_id}\n\n问题：{question}\n\n项目上下文：");
    if references.is_empty() {
        prompt.push_str("\n无可用引用。请明确说明上下文不足。");
        return prompt;
    }

    for (index, reference) in references.iter().enumerate() {
        prompt.push_str(&format!(
            "\n[{}] {} {}（{}:{}，来源：{}，分数：{:.2}）\n{}",
            index + 1,
            reference.entity_type,
            reference.title,
            reference.entity_type,
            reference.entity_id,
            reference.source,
            reference.score,
            reference.snippet
        ));
    }
    prompt
}

#[cfg(test)]
mod tests {
    use crate::ai::settings::{AiProviderKind, AiProviderSettingsView};

    use super::{build_agent_user_prompt, resolve_openai_agent_provider, AgentAskReference};

    fn reference(overrides: impl FnOnce(&mut AgentAskReference)) -> AgentAskReference {
        let mut reference = AgentAskReference {
            entity_type: "entry".into(),
            entity_id: "entry_1".into(),
            title: "月光阔剑".into(),
            snippet: "潮汐能武器，首次用于围城战。".into(),
            source: "keyword".into(),
            score: 2.0,
        };
        overrides(&mut reference);
        reference
    }

    fn openai_settings(enabled: bool) -> AiProviderSettingsView {
        AiProviderSettingsView {
            kind: AiProviderKind::OpenAiCompatible,
            base_url: Some("https://llm.example/v1".into()),
            api_key_preview: None,
            has_api_key: false,
            chat_model: Some("story-chat".into()),
            embedding_model: None,
            command_template: None,
            enabled,
        }
    }

    #[test]
    fn ai_agent_prompt_includes_question_and_numbered_references() {
        let prompt = build_agent_user_prompt(
            "project_1",
            "月光阔剑有什么风险？",
            &[
                reference(|_| {}),
                reference(|item| {
                    item.entity_type = "event".into();
                    item.entity_id = "event_1".into();
                    item.title = "围城战".into();
                    item.snippet = "围城战暴露了能源补给问题。".into();
                    item.source = "semantic".into();
                    item.score = 0.78;
                }),
            ],
        );

        assert!(prompt.contains("项目 ID：project_1"));
        assert!(prompt.contains("问题：月光阔剑有什么风险？"));
        assert!(prompt.contains("[1] entry 月光阔剑"));
        assert!(prompt.contains("潮汐能武器"));
        assert!(prompt.contains("[2] event 围城战"));
        assert!(prompt.contains("来源：semantic"));
    }

    #[test]
    fn ai_agent_prompt_marks_missing_context() {
        let prompt = build_agent_user_prompt("project_1", "谁掌握能源？", &[]);

        assert!(prompt.contains("无可用引用"));
        assert!(prompt.contains("上下文不足"));
    }

    #[test]
    fn ai_agent_provider_config_requires_enabled_openai_provider() {
        let error =
            resolve_openai_agent_provider(vec![openai_settings(false)], Some("sk-test".into()))
                .expect_err("disabled provider should fail");

        assert_eq!(error, "请先在 AI 设置中启用 OpenAI 兼容接口");
    }

    #[test]
    fn ai_agent_provider_config_requires_api_key() {
        let error = resolve_openai_agent_provider(vec![openai_settings(true)], None)
            .expect_err("missing api key should fail");

        assert_eq!(error, "请先保存 OpenAI 兼容接口的接口密钥");
    }
}
