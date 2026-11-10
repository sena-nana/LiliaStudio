use std::time::Duration;

use serde_json::Value;

use crate::ai::{
    cli_provider::{
        build_cli_invocation, classify_cli_failure, map_cli_process_error, CliInvocation,
        CliProviderError,
    },
    process_runner::{ProcessRunOutput, ProcessRunSpec, ProcessRunner},
};

pub const CLAUDE_TEST_PROMPT: &str = "Return the word ok.";
pub const CLAUDE_OUTPUT_FORMAT: &str = "json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeCliConfig {
    pub command_template: String,
    pub workspace: String,
    pub max_turns: u32,
    pub output_format: String,
    pub timeout: Duration,
}

pub struct ClaudeCliProvider<'a, R>
where
    R: ProcessRunner + ?Sized,
{
    config: ClaudeCliConfig,
    runner: &'a R,
}

impl<'a, R> ClaudeCliProvider<'a, R>
where
    R: ProcessRunner + ?Sized,
{
    pub fn new(config: ClaudeCliConfig, runner: &'a R) -> Self {
        Self { config, runner }
    }

    pub fn check_available(&self) -> Result<(), CliProviderError> {
        let invocation = self.build_invocation(CLAUDE_TEST_PROMPT)?;
        let output = self
            .runner
            .run(ProcessRunSpec {
                program: invocation.program,
                args: vec!["--help".into()],
                working_dir: None,
                timeout: self.config.timeout,
            })
            .map_err(|error| map_cli_process_error("Claude", "claude", error))?;
        if output.exit_code == Some(0) {
            Ok(())
        } else {
            Err(classify_cli_failure(
                "Claude",
                output,
                "Claude CLI 可用性检测失败",
            ))
        }
    }

    pub fn test_prompt(&self) -> Result<String, CliProviderError> {
        self.check_available()?;
        self.run_prompt(CLAUDE_TEST_PROMPT)
    }

    pub fn run_prompt(&self, prompt: &str) -> Result<String, CliProviderError> {
        let invocation = self.build_invocation(prompt)?;
        let output = self
            .runner
            .run(ProcessRunSpec {
                program: invocation.program,
                args: invocation.args,
                working_dir: Some(self.config.workspace.clone()),
                timeout: self.config.timeout,
            })
            .map_err(|error| map_cli_process_error("Claude", "claude", error))?;

        if output.exit_code == Some(0) {
            extract_claude_text(output)
        } else {
            Err(classify_cli_failure(
                "Claude",
                output,
                "Claude CLI 执行失败",
            ))
        }
    }

    fn build_invocation(&self, prompt: &str) -> Result<CliInvocation, CliProviderError> {
        build_cli_invocation(
            &self.config.command_template,
            &self.config.workspace,
            prompt,
            self.config.max_turns,
            &self.config.output_format,
        )
        .map_err(|error| {
            CliProviderError::execution_failed(format!("Claude CLI 命令模板无效：{error}"), None)
        })
    }
}

pub fn parse_claude_output(stdout: &str) -> Result<String, CliProviderError> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(CliProviderError::execution_failed(
            "Claude CLI 未返回可显示内容",
            None,
        ));
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return Ok(trimmed.to_string());
    };
    extract_text_from_json(&value).ok_or_else(|| {
        CliProviderError::execution_failed("Claude CLI JSON 输出中未找到文本内容", None)
    })
}

fn extract_claude_text(output: ProcessRunOutput) -> Result<String, CliProviderError> {
    let stdout = output.stdout.trim();
    if !stdout.is_empty() {
        return parse_claude_output(stdout);
    }
    let stderr = output.stderr.trim();
    if !stderr.is_empty() {
        return Ok(stderr.to_string());
    }
    Err(CliProviderError::execution_failed(
        "Claude CLI 未返回可显示内容",
        output.exit_code,
    ))
}

fn extract_text_from_json(value: &Value) -> Option<String> {
    for key in ["result", "text", "content"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }
    }

    let content = value.get("content")?.as_array()?;
    let text = content
        .iter()
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .filter(|part| !part.trim().is_empty())
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
