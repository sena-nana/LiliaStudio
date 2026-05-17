use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const SETTINGS_FILE_NAME: &str = "ai-provider-settings.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderKind {
    OpenAiCompatible,
    CodexCli,
    ClaudeCli,
}

impl AiProviderKind {
    fn secret_slug(self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "openAiCompatibleApiKey",
            Self::CodexCli => "codexCliApiKey",
            Self::ClaudeCli => "claudeCliApiKey",
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderSettingsView {
    pub kind: AiProviderKind,
    pub base_url: Option<String>,
    pub api_key_preview: Option<String>,
    pub has_api_key: bool,
    pub chat_model: Option<String>,
    pub embedding_model: Option<String>,
    pub command_template: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderSettingsDraft {
    pub kind: AiProviderKind,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub clear_api_key: bool,
    pub chat_model: Option<String>,
    pub embedding_model: Option<String>,
    pub command_template: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StoredAiProviderSettings {
    kind: AiProviderKind,
    base_url: Option<String>,
    has_api_key: bool,
    chat_model: Option<String>,
    embedding_model: Option<String>,
    command_template: Option<String>,
    enabled: bool,
}

pub trait SecretStore {
    fn read_secret(&self, name: &str) -> Result<Option<String>, String>;
    fn write_secret(&self, name: &str, secret: &str) -> Result<(), String>;
    fn delete_secret(&self, name: &str) -> Result<(), String>;
}

pub fn load_provider_settings(
    config_dir: &Path,
    secret_store: &impl SecretStore,
) -> Result<Vec<AiProviderSettingsView>, String> {
    let stored = read_stored_provider_settings(config_dir)?;
    stored
        .into_iter()
        .map(|provider| provider.into_view(secret_store))
        .collect()
}

pub fn save_provider_settings(
    config_dir: &Path,
    drafts: Vec<AiProviderSettingsDraft>,
    secret_store: &impl SecretStore,
) -> Result<Vec<AiProviderSettingsView>, String> {
    let mut stored = read_stored_provider_settings(config_dir)?;

    for draft in drafts {
        let index = stored
            .iter()
            .position(|provider| provider.kind == draft.kind)
            .unwrap_or_else(|| {
                stored.push(default_stored_provider(draft.kind));
                stored.len() - 1
            });

        let provider = &mut stored[index];
        provider.base_url = normalize_optional(draft.base_url);
        provider.chat_model = normalize_optional(draft.chat_model);
        provider.embedding_model = normalize_optional(draft.embedding_model);
        provider.command_template = normalize_optional(draft.command_template);
        provider.enabled = draft.enabled;

        let secret_name = api_key_secret_name(&draft.kind);
        if draft.clear_api_key {
            secret_store.delete_secret(&secret_name)?;
            provider.has_api_key = false;
        } else if let Some(api_key) = normalize_optional(draft.api_key) {
            secret_store.write_secret(&secret_name, &api_key)?;
            provider.has_api_key = true;
        }
    }

    stored = merge_with_defaults(stored);
    write_stored_provider_settings(config_dir, &stored)?;
    load_provider_settings(config_dir, secret_store)
}

pub fn api_key_secret_name(kind: &AiProviderKind) -> String {
    format!("ameya_{}", kind.secret_slug())
}

pub fn mask_secret(secret: Option<&str>) -> String {
    let Some(secret) = secret else {
        return String::new();
    };
    if secret.chars().count() <= 8 {
        return "********".into();
    }
    let prefix: String = secret.chars().take(3).collect();
    let suffix_chars: Vec<char> = secret.chars().rev().take(4).collect();
    let suffix: String = suffix_chars.into_iter().rev().collect();
    format!("{prefix}-********{suffix}").replace("--", "-")
}

fn provider_settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join(SETTINGS_FILE_NAME)
}

fn read_stored_provider_settings(
    config_dir: &Path,
) -> Result<Vec<StoredAiProviderSettings>, String> {
    let path = provider_settings_path(config_dir);
    if !path.exists() {
        return Ok(default_stored_providers());
    }

    let config_text = fs::read_to_string(&path)
        .map_err(|error| format!("读取 AI Provider 设置失败：{error}"))?;
    if config_text.trim().is_empty() {
        return Ok(default_stored_providers());
    }

    let stored: Vec<StoredAiProviderSettings> = serde_json::from_str(&config_text)
        .map_err(|error| format!("解析 AI Provider 设置失败：{error}"))?;
    Ok(merge_with_defaults(stored))
}

fn write_stored_provider_settings(
    config_dir: &Path,
    providers: &[StoredAiProviderSettings],
) -> Result<(), String> {
    fs::create_dir_all(config_dir)
        .map_err(|error| format!("创建 AI Provider 设置目录失败：{error}"))?;
    let path = provider_settings_path(config_dir);
    let config_text = serde_json::to_string_pretty(providers)
        .map_err(|error| format!("序列化 AI Provider 设置失败：{error}"))?;
    fs::write(path, config_text).map_err(|error| format!("保存 AI Provider 设置失败：{error}"))
}

fn default_stored_providers() -> Vec<StoredAiProviderSettings> {
    vec![
        default_stored_provider(AiProviderKind::OpenAiCompatible),
        StoredAiProviderSettings::from_config(AiProviderConfig::codex_cli()),
        StoredAiProviderSettings::from_config(AiProviderConfig::claude_cli()),
    ]
}

fn default_stored_provider(kind: AiProviderKind) -> StoredAiProviderSettings {
    match kind {
        AiProviderKind::OpenAiCompatible => StoredAiProviderSettings {
            kind,
            base_url: None,
            has_api_key: false,
            chat_model: None,
            embedding_model: None,
            command_template: None,
            enabled: false,
        },
        AiProviderKind::CodexCli => StoredAiProviderSettings::from_config(AiProviderConfig::codex_cli()),
        AiProviderKind::ClaudeCli => StoredAiProviderSettings::from_config(AiProviderConfig::claude_cli()),
    }
}

fn merge_with_defaults(stored: Vec<StoredAiProviderSettings>) -> Vec<StoredAiProviderSettings> {
    let mut merged = default_stored_providers();
    for provider in stored {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.kind == provider.kind)
        {
            *existing = provider;
        } else {
            merged.push(provider);
        }
    }
    merged
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

impl StoredAiProviderSettings {
    fn from_config(config: AiProviderConfig) -> Self {
        Self {
            kind: config.kind,
            base_url: config.base_url,
            has_api_key: config.api_key.as_ref().is_some_and(|secret| !secret.is_empty()),
            chat_model: config.chat_model,
            embedding_model: config.embedding_model,
            command_template: config.command_template,
            enabled: config.enabled,
        }
    }

    fn into_view(
        self,
        secret_store: &impl SecretStore,
    ) -> Result<AiProviderSettingsView, String> {
        let secret = if self.has_api_key {
            secret_store.read_secret(&api_key_secret_name(&self.kind))?
        } else {
            None
        };
        let api_key_preview = secret
            .as_deref()
            .filter(|secret| !secret.is_empty())
            .map(|secret| mask_secret(Some(secret)));
        let has_api_key = api_key_preview.is_some();

        Ok(AiProviderSettingsView {
            kind: self.kind,
            base_url: self.base_url,
            api_key_preview,
            has_api_key,
            chat_model: self.chat_model,
            embedding_model: self.embedding_model,
            command_template: self.command_template,
            enabled: self.enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use super::{
        api_key_secret_name, load_provider_settings, save_provider_settings, AiProviderKind,
        AiProviderSettingsDraft, SecretStore,
    };

    #[derive(Default)]
    struct MemorySecretStore {
        secrets: Mutex<HashMap<String, String>>,
    }

    impl SecretStore for MemorySecretStore {
        fn read_secret(&self, name: &str) -> Result<Option<String>, String> {
            Ok(self.secrets.lock().expect("secret lock").get(name).cloned())
        }

        fn write_secret(&self, name: &str, secret: &str) -> Result<(), String> {
            self.secrets
                .lock()
                .expect("secret lock")
                .insert(name.to_string(), secret.to_string());
            Ok(())
        }

        fn delete_secret(&self, name: &str) -> Result<(), String> {
            self.secrets.lock().expect("secret lock").remove(name);
            Ok(())
        }
    }

    #[test]
    fn loads_default_provider_settings_when_config_is_missing() {
        let config_dir = tempfile::tempdir().expect("temp config dir");
        let store = MemorySecretStore::default();

        let settings = load_provider_settings(config_dir.path(), &store).expect("settings load");

        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].kind, AiProviderKind::OpenAiCompatible);
        assert!(!settings[0].enabled);
        assert!(!settings[0].has_api_key);
        assert_eq!(settings[1].kind, AiProviderKind::CodexCli);
        assert!(
            settings[1]
                .command_template
                .as_deref()
                .expect("codex template")
                .contains("codex exec")
        );
        assert_eq!(settings[2].kind, AiProviderKind::ClaudeCli);
    }

    #[test]
    fn saves_non_secret_config_and_returns_masked_key_preview() {
        let config_dir = tempfile::tempdir().expect("temp config dir");
        let store = MemorySecretStore::default();
        let drafts = vec![AiProviderSettingsDraft {
            kind: AiProviderKind::OpenAiCompatible,
            base_url: Some("https://llm.example/v1".into()),
            api_key: Some("sk-live-secret-1234".into()),
            clear_api_key: false,
            chat_model: Some("story-chat".into()),
            embedding_model: Some("story-embed".into()),
            command_template: None,
            enabled: true,
        }];

        let settings = save_provider_settings(config_dir.path(), drafts, &store).expect("settings save");

        assert_eq!(settings[0].api_key_preview.as_deref(), Some("sk-********1234"));
        assert!(settings[0].has_api_key);
        let config_text = std::fs::read_to_string(config_dir.path().join("ai-provider-settings.json"))
            .expect("config file exists");
        assert!(config_text.contains("llm.example"));
        assert!(!config_text.contains("sk-live-secret-1234"));
        assert_eq!(
            store
                .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))
                .expect("secret read")
                .as_deref(),
            Some("sk-live-secret-1234")
        );
    }

    #[test]
    fn clear_api_key_removes_existing_secret() {
        let config_dir = tempfile::tempdir().expect("temp config dir");
        let store = MemorySecretStore::default();
        let secret_name = api_key_secret_name(&AiProviderKind::OpenAiCompatible);
        store
            .write_secret(&secret_name, "sk-old-secret")
            .expect("seed secret");
        let drafts = vec![AiProviderSettingsDraft {
            kind: AiProviderKind::OpenAiCompatible,
            base_url: Some("https://llm.example/v1".into()),
            api_key: None,
            clear_api_key: true,
            chat_model: Some("story-chat".into()),
            embedding_model: None,
            command_template: None,
            enabled: true,
        }];

        let settings = save_provider_settings(config_dir.path(), drafts, &store).expect("settings save");

        assert!(!settings[0].has_api_key);
        assert_eq!(settings[0].api_key_preview, None);
        assert_eq!(
            store
                .read_secret(&api_key_secret_name(&AiProviderKind::OpenAiCompatible))
                .expect("secret read"),
            None
        );
    }
}
