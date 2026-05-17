use std::{sync::Mutex, time::Duration};

use ameya_lib::ai::{
    claude_cli::{parse_claude_output, ClaudeCliConfig, ClaudeCliProvider, CLAUDE_OUTPUT_FORMAT},
    cli_provider::CliProviderErrorCode,
    process_runner::{
        ProcessRunError, ProcessRunErrorCode, ProcessRunOutput, ProcessRunSpec, ProcessRunner,
    },
    settings::AiProviderConfig,
};

struct FakeRunner {
    requests: Mutex<Vec<ProcessRunSpec>>,
    responses: Mutex<Vec<Result<ProcessRunOutput, ProcessRunError>>>,
}

impl FakeRunner {
    fn new(responses: Vec<Result<ProcessRunOutput, ProcessRunError>>) -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
            responses: Mutex::new(responses),
        }
    }

    fn requests(&self) -> Vec<ProcessRunSpec> {
        self.requests.lock().expect("request lock").clone()
    }
}

impl ProcessRunner for FakeRunner {
    fn run(&self, spec: ProcessRunSpec) -> Result<ProcessRunOutput, ProcessRunError> {
        self.requests.lock().expect("request lock").push(spec);
        self.responses.lock().expect("response lock").remove(0)
    }
}

#[test]
fn claude_default_template_uses_print_mode_json_output() {
    let template = AiProviderConfig::claude_cli()
        .command_template
        .expect("claude template");

    assert!(template.starts_with("claude -p"));
    assert!(template.contains("--output-format json"));
}

#[test]
fn claude_availability_check_runs_claude_help() {
    let runner = FakeRunner::new(vec![Ok(ok_output("Claude help"))]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    provider.check_available().expect("claude help succeeds");

    let requests = runner.requests();
    assert_eq!(requests[0].program, "claude");
    assert_eq!(requests[0].args, vec!["--help"]);
}

#[test]
fn claude_availability_check_uses_program_from_custom_template() {
    let runner = FakeRunner::new(vec![Ok(ok_output("Claude help"))]);
    let mut config = default_config();
    config.command_template =
        r#""C:/Tools/claude.exe" -p "{prompt}" --output-format json --cwd "{workspace}""#.into();
    let provider = ClaudeCliProvider::new(config, &runner);

    provider.check_available().expect("claude help succeeds");

    let requests = runner.requests();
    assert_eq!(requests[0].program, "C:/Tools/claude.exe");
    assert_eq!(requests[0].args, vec!["--help"]);
}

#[test]
fn claude_prompt_uses_rendered_template_and_parses_json_result() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Claude help")),
        Ok(ok_output(r#"{"result":"ok"}"#)),
    ]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    let text = provider.test_prompt().expect("claude prompt succeeds");

    assert_eq!(text, "ok");
    let requests = runner.requests();
    assert_eq!(requests[1].program, "claude");
    assert_eq!(requests[1].args[0], "-p");
    assert!(requests[1]
        .args
        .contains(&"Return the word ok.".to_string()));
}

#[test]
fn parse_claude_output_supports_text_and_content_array() {
    assert_eq!(parse_claude_output(r#"{"text":"ok"}"#).unwrap(), "ok");
    assert_eq!(
        parse_claude_output(r#"{"content":[{"type":"text","text":"first"},{"type":"tool_use","name":"x"},{"text":"second"}]}"#)
            .unwrap(),
        "first\nsecond"
    );
}

#[test]
fn parse_claude_output_preserves_plain_stdout_when_json_parse_fails() {
    assert_eq!(parse_claude_output("plain output").unwrap(), "plain output");
}

#[test]
fn claude_provider_classifies_missing_cli() {
    let runner = FakeRunner::new(vec![Err(ProcessRunError {
        code: ProcessRunErrorCode::SpawnFailed,
        message: "program not found".into(),
    })]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    let error = provider.check_available().expect_err("missing CLI");

    assert_eq!(error.code, CliProviderErrorCode::MissingCli);
}

#[test]
fn claude_provider_classifies_auth_failure() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Claude help")),
        Ok(failed_output("please login to continue")),
    ]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("auth failure");

    assert_eq!(error.code, CliProviderErrorCode::AuthFailed);
}

#[test]
fn claude_provider_classifies_execution_failure() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Claude help")),
        Ok(failed_output("unexpected execution error")),
    ]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("execution failure");

    assert_eq!(error.code, CliProviderErrorCode::ExecutionFailed);
    assert_eq!(error.exit_code, Some(1));
}

#[test]
fn claude_provider_classifies_timeout() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Claude help")),
        Err(ProcessRunError {
            code: ProcessRunErrorCode::TimedOut,
            message: "process timed out after 50 ms".into(),
        }),
    ]);
    let provider = ClaudeCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("timeout");

    assert_eq!(error.code, CliProviderErrorCode::TimedOut);
}

fn default_config() -> ClaudeCliConfig {
    ClaudeCliConfig {
        command_template: AiProviderConfig::claude_cli()
            .command_template
            .expect("claude template"),
        workspace: "D:/PROJECT/workspace/ameya".into(),
        max_turns: 1,
        output_format: CLAUDE_OUTPUT_FORMAT.into(),
        timeout: Duration::from_secs(5),
    }
}

fn ok_output(stdout: &str) -> ProcessRunOutput {
    ProcessRunOutput {
        stdout: stdout.into(),
        stderr: String::new(),
        exit_code: Some(0),
    }
}

fn failed_output(stderr: &str) -> ProcessRunOutput {
    ProcessRunOutput {
        stdout: String::new(),
        stderr: stderr.into(),
        exit_code: Some(1),
    }
}
