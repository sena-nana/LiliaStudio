use ameya_lib::{
    ai::{
        cli_provider::{build_cli_invocation, render_command_template, split_command_line},
        openai_compatible::{
            chat_url, embeddings_url, parse_chat_content, parse_embeddings, ChatMessage,
            OpenAiCompatibleClient, OpenAiRequest, OpenAiResponse, OpenAiTransport,
            ProviderErrorCode,
        },
        process_runner::{run_process, ProcessRunErrorCode, ProcessRunSpec},
        settings::{mask_secret, AiProviderConfig},
    },
    vector::{chunking::chunk_text, search::cosine_similarity},
};
use std::{io::Write, sync::Mutex, time::Duration};

struct FakeTransport {
    response: OpenAiResponse,
    requests: Mutex<Vec<OpenAiRequest>>,
}

impl FakeTransport {
    fn new(response: OpenAiResponse) -> Self {
        Self {
            response,
            requests: Mutex::new(Vec::new()),
        }
    }
}

impl OpenAiTransport for FakeTransport {
    fn post_json(
        &self,
        request: OpenAiRequest,
    ) -> Result<OpenAiResponse, ameya_lib::ai::openai_compatible::ProviderError> {
        self.requests.lock().expect("request lock").push(request);
        Ok(self.response.clone())
    }
}

#[test]
fn renders_cli_command_template_without_losing_quoted_prompt() {
    let rendered = render_command_template(
        r#"codex exec --cd "{workspace}" "{prompt}""#,
        "D:/PROJECT/workspace/ameya",
        "请审计角色行为",
        3,
        "json",
    )
    .unwrap();
    let parts = split_command_line(&rendered).unwrap();

    assert_eq!(parts[0], "codex");
    assert_eq!(parts[1], "exec");
    assert!(parts.contains(&"D:/PROJECT/workspace/ameya".to_string()));
    assert!(parts.contains(&"请审计角色行为".to_string()));
}

#[test]
fn parses_openai_compatible_responses() {
    assert_eq!(
        chat_url("https://api.example.test/v1/"),
        "https://api.example.test/v1/chat/completions"
    );
    assert_eq!(
        embeddings_url("https://api.example.test/v1"),
        "https://api.example.test/v1/embeddings"
    );
    let chat = parse_chat_content(
        r#"{"choices":[{"message":{"role":"assistant","content":"审计完成"}}]}"#,
    )
    .unwrap();
    let embeddings = parse_embeddings(r#"{"data":[{"embedding":[1.0,0.0,0.5]}]}"#).unwrap();

    assert_eq!(chat, "审计完成");
    assert_eq!(embeddings[0], vec![1.0, 0.0, 0.5]);
}

#[test]
fn builds_cli_invocation_as_program_and_argument_array() {
    let invocation = build_cli_invocation(
        r#"codex exec --cd "{workspace}" --max-turns {max_turns} --format {output_format} "{prompt}""#,
        "D:/PROJECT/workspace/ameya",
        "请审计角色行为",
        3,
        "json",
    )
    .unwrap();

    assert_eq!(invocation.program, "codex");
    assert_eq!(
        invocation.args,
        vec![
            "exec",
            "--cd",
            "D:/PROJECT/workspace/ameya",
            "--max-turns",
            "3",
            "--format",
            "json",
            "请审计角色行为"
        ]
    );
}

#[test]
fn process_runner_captures_stdout_stderr_and_exit_code() {
    let spec = ProcessRunSpec {
        program: current_test_binary(),
        args: vec!["process_runner_child_probe".into(), "--nocapture".into()],
        working_dir: None,
        timeout: Duration::from_secs(5),
    };

    let output = run_process(spec).expect("process output");

    assert_eq!(output.exit_code, Some(7));
    assert!(output.stdout.contains("probe stdout"));
    assert!(output.stderr.contains("probe stderr"));
}

#[test]
fn process_runner_returns_structured_timeout_error() {
    let spec = ProcessRunSpec {
        program: current_test_binary(),
        args: vec!["process_runner_child_sleep".into()],
        working_dir: None,
        timeout: Duration::from_millis(50),
    };

    let error = run_process(spec).expect_err("sleeping process should time out");

    assert_eq!(error.code, ProcessRunErrorCode::TimedOut);
    assert!(error.message.contains("timed out"));
}

#[test]
fn openai_provider_chat_sends_bearer_token_and_extracts_content() {
    let transport = FakeTransport::new(OpenAiResponse {
        status: 200,
        body: r#"{"choices":[{"message":{"role":"assistant","content":"可用"}}]}"#.into(),
    });
    let client = OpenAiCompatibleClient::new(
        "https://api.example.test/v1".into(),
        "sk-test".into(),
        transport,
    );

    let content = client
        .chat(
            "story-chat",
            vec![ChatMessage {
                role: "user".into(),
                content: "ping".into(),
            }],
            0.2,
        )
        .expect("chat succeeds");

    assert_eq!(content, "可用");
    let requests = client.transport().requests.lock().expect("request lock");
    assert_eq!(
        requests[0].url,
        "https://api.example.test/v1/chat/completions"
    );
    assert_eq!(requests[0].bearer_token, "sk-test");
    assert!(requests[0].body.contains("\"model\":\"story-chat\""));
}

#[test]
fn openai_provider_embeddings_returns_vectors_and_dimension() {
    let transport = FakeTransport::new(OpenAiResponse {
        status: 200,
        body: r#"{"data":[{"embedding":[1.0,0.0]},{"embedding":[0.0,1.0]}]}"#.into(),
    });
    let client = OpenAiCompatibleClient::new(
        "https://api.example.test/v1".into(),
        "sk-test".into(),
        transport,
    );

    let result = client
        .embeddings("story-embed", vec!["月光剑".into(), "潮汐城".into()])
        .expect("embedding succeeds");

    assert_eq!(result.dimension, 2);
    assert_eq!(result.vectors.len(), 2);
}

#[test]
fn openai_provider_classifies_auth_failures() {
    let transport = FakeTransport::new(OpenAiResponse {
        status: 401,
        body: r#"{"error":{"message":"invalid api key"}}"#.into(),
    });
    let client = OpenAiCompatibleClient::new(
        "https://api.example.test/v1".into(),
        "bad-key".into(),
        transport,
    );

    let error = client
        .chat(
            "story-chat",
            vec![ChatMessage {
                role: "user".into(),
                content: "ping".into(),
            }],
            0.2,
        )
        .expect_err("auth should fail");

    assert_eq!(error.code, ProviderErrorCode::AuthFailed);
    assert_eq!(error.status, Some(401));
    assert!(error.message.contains("invalid api key"));
}

#[test]
fn masks_provider_secrets_for_display() {
    let config = AiProviderConfig::openai_compatible(
        "https://api.example.test/v1".into(),
        "sk-1234567890abcdef".into(),
        "chat-model".into(),
        "embedding-model".into(),
    );

    assert_eq!(mask_secret(config.api_key.as_deref()), "sk-********cdef");
}

#[test]
fn chunks_text_by_stable_boundaries() {
    let chunks = chunk_text("第一段。\n\n第二段包含更多世界观设定。\n\n第三段。", 12);

    assert_eq!(chunks.len(), 4);
    assert_eq!(chunks[0].text, "第一段。");
    assert_eq!(chunks[1].ordinal, 1);
    assert!(chunks[1].text.starts_with("第二段"));
    assert_eq!(chunks[3].text, "第三段。");
}

#[test]
fn cosine_similarity_scores_identical_vectors_highest() {
    let same = cosine_similarity(&[1.0, 0.0, 1.0], &[1.0, 0.0, 1.0]).unwrap();
    let different = cosine_similarity(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]).unwrap();

    assert!(same > 0.99);
    assert_eq!(different, 0.0);
}

fn current_test_binary() -> String {
    std::env::current_exe()
        .expect("current test exe")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn process_runner_child_probe() {
    if std::env::args().any(|arg| arg == "process_runner_child_probe") {
        println!("probe stdout");
        eprintln!("probe stderr");
        std::io::stdout().flush().expect("flush stdout");
        std::io::stderr().flush().expect("flush stderr");
        std::process::exit(7);
    }
}

#[test]
fn process_runner_child_sleep() {
    if std::env::args().any(|arg| arg == "process_runner_child_sleep") {
        std::thread::sleep(Duration::from_secs(5));
    }
}
