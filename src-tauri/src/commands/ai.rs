use crate::ai::settings::AiProviderConfig;

#[tauri::command]
pub fn default_ai_providers() -> Vec<AiProviderConfig> {
    vec![
        AiProviderConfig::codex_cli(),
        AiProviderConfig::claude_cli(),
    ]
}
