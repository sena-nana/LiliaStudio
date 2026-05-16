use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderKind {
    OpenAiCompatible,
    CodexCli,
    ClaudeCli,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub kind: AiProviderKind,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub chat_model: Option<String>,
    pub embedding_model: Option<String>,
    pub command_template: Option<String>,
    pub enabled: bool,
}

impl AiProviderConfig {
    pub fn openai_compatible(
        base_url: String,
        api_key: String,
        chat_model: String,
        embedding_model: String,
    ) -> Self {
        Self {
            kind: AiProviderKind::OpenAiCompatible,
            base_url: Some(base_url),
            api_key: Some(api_key),
            chat_model: Some(chat_model),
            embedding_model: Some(embedding_model),
            command_template: None,
            enabled: true,
        }
    }

    pub fn codex_cli() -> Self {
        Self {
            kind: AiProviderKind::CodexCli,
            base_url: None,
            api_key: None,
            chat_model: None,
            embedding_model: None,
            command_template: Some(
                r#"codex exec --cd "{workspace}" --ask-for-approval never --sandbox workspace-write "{prompt}""#
                    .into(),
            ),
            enabled: true,
        }
    }

    pub fn claude_cli() -> Self {
        Self {
            kind: AiProviderKind::ClaudeCli,
            base_url: None,
            api_key: None,
            chat_model: None,
            embedding_model: None,
            command_template: Some(
                r#"claude -p "{prompt}" --output-format json --cwd "{workspace}" --max-turns {max_turns}"#
                    .into(),
            ),
            enabled: true,
        }
    }
}

pub fn mask_secret(secret: Option<&str>) -> String {
    let Some(secret) = secret else {
        return String::new();
    };
    if secret.len() <= 8 {
        return "********".into();
    }
    let prefix = &secret[..3.min(secret.len())];
    let suffix = &secret[secret.len() - 4..];
    format!("{prefix}-********{suffix}").replace("--", "-")
}
