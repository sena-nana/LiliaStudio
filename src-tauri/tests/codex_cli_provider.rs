use std::{sync::Mutex, time::Duration};

use ameya_lib::ai::{
    cli_provider::{CliProviderErrorCode, DEFAULT_CLI_OUTPUT_FORMAT},
    codex_cli::{CodexCliConfig, CodexCliProvider},
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
fn codex_default_template_uses_codex_exec() {
    let template = AiProviderConfig::codex_cli()
        .command_template
        .expect("codex template");

    assert!(template.starts_with("codex exec"));
}

#[test]
fn codex_availability_check_runs_codex_help() {
    let runner = FakeRunner::new(vec![Ok(ok_output("Codex help"))]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    provider.check_available().expect("codex help succeeds");

    let requests = runner.requests();
    assert_eq!(requests[0].program, "codex");
    assert_eq!(requests[0].args, vec!["--help"]);
}

#[test]
fn codex_availability_check_uses_program_from_custom_template() {
    let runner = FakeRunner::new(vec![Ok(ok_output("Codex help"))]);
    let mut config = default_config();
    config.command_template = r#""C:/Tools/codex.exe" exec --cd "{workspace}" "{prompt}""#.into();
    let provider = CodexCliProvider::new(config, &runner);

    provider.check_available().expect("codex help succeeds");

    let requests = runner.requests();
    assert_eq!(requests[0].program, "C:/Tools/codex.exe");
    assert_eq!(requests[0].args, vec!["--help"]);
}

#[test]
fn codex_prompt_uses_rendered_template_and_returns_stdout_text() {
    let runner = FakeRunner::new(vec![Ok(ok_output("Codex help")), Ok(ok_output("ok"))]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    let text = provider.test_prompt().expect("codex prompt succeeds");

    assert_eq!(text, "ok");
    let requests = runner.requests();
    assert_eq!(requests[1].program, "codex");
    assert_eq!(requests[1].args[0], "exec");
    assert!(requests[1]
        .args
        .contains(&"Return the word ok.".to_string()));
}

#[test]
fn codex_provider_classifies_missing_cli() {
    let runner = FakeRunner::new(vec![Err(ProcessRunError {
        code: ProcessRunErrorCode::SpawnFailed,
        message: "program not found".into(),
    })]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    let error = provider.check_available().expect_err("missing CLI");

    assert_eq!(error.code, CliProviderErrorCode::MissingCli);
}

#[test]
fn codex_provider_classifies_auth_failure() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Codex help")),
        Ok(failed_output("please login to continue")),
    ]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("auth failure");

    assert_eq!(error.code, CliProviderErrorCode::AuthFailed);
}

#[test]
fn codex_provider_classifies_execution_failure() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Codex help")),
        Ok(failed_output("unexpected execution error")),
    ]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("execution failure");

    assert_eq!(error.code, CliProviderErrorCode::ExecutionFailed);
    assert_eq!(error.exit_code, Some(1));
}

#[test]
fn codex_provider_classifies_timeout() {
    let runner = FakeRunner::new(vec![
        Ok(ok_output("Codex help")),
        Err(ProcessRunError {
            code: ProcessRunErrorCode::TimedOut,
            message: "process timed out after 50 ms".into(),
        }),
    ]);
    let provider = CodexCliProvider::new(default_config(), &runner);

    let error = provider.test_prompt().expect_err("timeout");

    assert_eq!(error.code, CliProviderErrorCode::TimedOut);
}

fn default_config() -> CodexCliConfig {
    CodexCliConfig {
        command_template: AiProviderConfig::codex_cli()
            .command_template
            .expect("codex template"),
        workspace: "D:/PROJECT/workspace/ameya".into(),
        max_turns: 1,
        output_format: DEFAULT_CLI_OUTPUT_FORMAT.into(),
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
