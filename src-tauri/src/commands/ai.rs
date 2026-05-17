use tauri::Manager;

use crate::{
    ai::settings::{
        load_provider_settings, save_provider_settings, AiProviderConfig, AiProviderSettingsDraft,
        AiProviderSettingsView,
    },
    windows::credential::WindowsCredentialStore,
};

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
