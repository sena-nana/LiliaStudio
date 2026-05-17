use std::time::Duration;

use tauri::Manager;

use crate::{
    ai::{
        claude_cli::{ClaudeCliConfig, ClaudeCliProvider, CLAUDE_OUTPUT_FORMAT},
        cli_provider::{CliProviderError, DEFAULT_CLI_OUTPUT_FORMAT},
        codex_cli::{CodexCliConfig, CodexCliProvider},
        process_runner::StdProcessRunner,
        settings::{load_provider_settings, AiProviderConfig, AiProviderKind},
    },
    windows::credential::WindowsCredentialStore,
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliProviderTestResult {
    pub ok: bool,
    pub message: String,
    pub error: Option<CliProviderError>,
    pub output: Option<String>,
}

#[tauri::command]
pub fn test_codex_cli_provider<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<CliProviderTestResult, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let workspace = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
    std::fs::create_dir_all(&workspace)
        .map_err(|error| format!("创建 Codex CLI 工作目录失败：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::CodexCli)
        .ok_or_else(|| "未找到 Codex CLI Provider 设置".to_string())?;
    let command_template = provider
        .command_template
        .or_else(|| AiProviderConfig::codex_cli().command_template)
        .ok_or_else(|| "未找到 Codex CLI 命令模板".to_string())?;
    let config = CodexCliConfig {
        command_template,
        workspace: workspace.to_string_lossy().into_owned(),
        max_turns: 1,
        output_format: DEFAULT_CLI_OUTPUT_FORMAT.into(),
        timeout: Duration::from_secs(60),
    };
    let runner = StdProcessRunner;
    let codex = CodexCliProvider::new(config, &runner);

    match codex.test_prompt() {
        Ok(output) => Ok(CliProviderTestResult {
            ok: true,
            message: "Codex CLI 测试调用成功".into(),
            error: None,
            output: Some(output),
        }),
        Err(error) => Ok(CliProviderTestResult {
            ok: false,
            message: error.message.clone(),
            error: Some(error),
            output: None,
        }),
    }
}

#[tauri::command]
pub fn test_claude_cli_provider<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<CliProviderTestResult, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;
    let workspace = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
    std::fs::create_dir_all(&workspace)
        .map_err(|error| format!("创建 Claude CLI 工作目录失败：{error}"))?;
    let settings = load_provider_settings(&config_dir, &WindowsCredentialStore)?;
    let provider = settings
        .into_iter()
        .find(|provider| provider.kind == AiProviderKind::ClaudeCli)
        .ok_or_else(|| "未找到 Claude CLI Provider 设置".to_string())?;
    let command_template = provider
        .command_template
        .or_else(|| AiProviderConfig::claude_cli().command_template)
        .ok_or_else(|| "未找到 Claude CLI 命令模板".to_string())?;
    let config = ClaudeCliConfig {
        command_template,
        workspace: workspace.to_string_lossy().into_owned(),
        max_turns: 1,
        output_format: CLAUDE_OUTPUT_FORMAT.into(),
        timeout: Duration::from_secs(60),
    };
    let runner = StdProcessRunner;
    let claude = ClaudeCliProvider::new(config, &runner);

    match claude.test_prompt() {
        Ok(output) => Ok(CliProviderTestResult {
            ok: true,
            message: "Claude CLI 测试调用成功".into(),
            error: None,
            output: Some(output),
        }),
        Err(error) => Ok(CliProviderTestResult {
            ok: false,
            message: error.message.clone(),
            error: Some(error),
            output: None,
        }),
    }
}
