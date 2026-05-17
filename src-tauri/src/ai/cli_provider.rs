use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_CLI_OUTPUT_FORMAT: &str = "text";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CliTemplateError {
    #[error("command template produced an empty command")]
    EmptyCommand,
    #[error("unterminated quote in command template")]
    UnterminatedQuote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInvocation {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CliProviderErrorCode {
    MissingCli,
    AuthFailed,
    ExecutionFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CliProviderError {
    pub code: CliProviderErrorCode,
    pub message: String,
    pub exit_code: Option<i32>,
}

impl CliProviderError {
    pub fn missing_cli(message: impl Into<String>) -> Self {
        Self {
            code: CliProviderErrorCode::MissingCli,
            message: message.into(),
            exit_code: None,
        }
    }

    pub fn auth_failed(message: impl Into<String>, exit_code: Option<i32>) -> Self {
        Self {
            code: CliProviderErrorCode::AuthFailed,
            message: message.into(),
            exit_code,
        }
    }

    pub fn execution_failed(message: impl Into<String>, exit_code: Option<i32>) -> Self {
        Self {
            code: CliProviderErrorCode::ExecutionFailed,
            message: message.into(),
            exit_code,
        }
    }

    pub fn timed_out(message: impl Into<String>) -> Self {
        Self {
            code: CliProviderErrorCode::TimedOut,
            message: message.into(),
            exit_code: None,
        }
    }
}

pub fn render_command_template(
    template: &str,
    workspace: &str,
    prompt: &str,
    max_turns: u32,
    output_format: &str,
) -> Result<String, CliTemplateError> {
    Ok(template
        .replace("{workspace}", workspace)
        .replace("{prompt}", prompt)
        .replace("{max_turns}", &max_turns.to_string())
        .replace("{output_format}", output_format))
}

pub fn build_cli_invocation(
    template: &str,
    workspace: &str,
    prompt: &str,
    max_turns: u32,
    output_format: &str,
) -> Result<CliInvocation, CliTemplateError> {
    let command = render_command_template(template, workspace, prompt, max_turns, output_format)?;
    let mut parts = split_command_line(&command)?;
    if parts.is_empty() {
        return Err(CliTemplateError::EmptyCommand);
    }
    let program = parts.remove(0);
    Ok(CliInvocation {
        program,
        args: parts,
    })
}

pub fn split_command_line(command: &str) -> Result<Vec<String>, CliTemplateError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '\\' if chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(CliTemplateError::UnterminatedQuote);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    Ok(parts)
}
