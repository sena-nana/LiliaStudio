use std::time::Duration;

use crate::ai::{
    cli_provider::{
        build_cli_invocation, classify_cli_failure, map_cli_process_error, CliInvocation,
        CliProviderError,
    },
    process_runner::{ProcessRunOutput, ProcessRunSpec, ProcessRunner},
};

pub const CODEX_TEST_PROMPT: &str = "Return the word ok.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexCliConfig {
    pub command_template: String,
    pub workspace: String,
    pub max_turns: u32,
    pub output_format: String,
    pub timeout: Duration,
}

pub struct CodexCliProvider<'a, R>
where
    R: ProcessRunner + ?Sized,
{
    config: CodexCliConfig,
    runner: &'a R,
}

impl<'a, R> CodexCliProvider<'a, R>
where
    R: ProcessRunner + ?Sized,
{
    pub fn new(config: CodexCliConfig, runner: &'a R) -> Self {
        Self { config, runner }
    }

    pub fn check_available(&self) -> Result<(), CliProviderError> {
        let invocation = self.build_invocation(CODEX_TEST_PROMPT)?;
        let output = self
            .runner
            .run(ProcessRunSpec {
                program: invocation.program,
                args: vec!["--help".into()],
                working_dir: None,
                timeout: self.config.timeout,
            })
            .map_err(|error| map_cli_process_error("Codex", "codex", error))?;
        if output.exit_code == Some(0) {
            Ok(())
        } else {
            Err(classify_cli_failure(
                "Codex",
                output,
                "Codex CLI 可用性检测失败",
            ))
        }
    }

    pub fn test_prompt(&self) -> Result<String, CliProviderError> {
        self.check_available()?;
        self.run_prompt(CODEX_TEST_PROMPT)
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
            .map_err(|error| map_cli_process_error("Codex", "codex", error))?;

        if output.exit_code == Some(0) {
            extract_codex_text(output)
        } else {
            Err(classify_cli_failure("Codex", output, "Codex CLI 执行失败"))
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
            CliProviderError::execution_failed(format!("Codex CLI 命令模板无效：{error}"), None)
        })
    }
}

fn extract_codex_text(output: ProcessRunOutput) -> Result<String, CliProviderError> {
    let stdout = output.stdout.trim();
    if !stdout.is_empty() {
        return Ok(stdout.to_string());
    }
    let stderr = output.stderr.trim();
    if !stderr.is_empty() {
        return Ok(stderr.to_string());
    }
    Err(CliProviderError::execution_failed(
        "Codex CLI 未返回可显示内容",
        output.exit_code,
    ))
}
