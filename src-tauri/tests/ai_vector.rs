use ameya_lib::{
    ai::{
        cli_provider::{render_command_template, split_command_line},
        openai_compatible::{chat_url, embeddings_url, parse_chat_content, parse_embeddings},
        settings::{mask_secret, AiProviderConfig},
    },
    vector::{chunking::chunk_text, search::cosine_similarity},
};

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
