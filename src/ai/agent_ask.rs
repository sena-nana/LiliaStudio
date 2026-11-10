//! Project-context Q&A over an OpenAI-compatible chat model.
//!
//! Ported from the Tauri command layer (`src-tauri/src/commands/ai.rs`) into
//! the core crate: the message assembly is pure and unit-tested here, while
//! callers (native host, Tauri shell) resolve the provider and invoke the
//! model through [`run_agent_ask`].

use crate::ai::openai_compatible::{ChatMessage, OpenAiCompatibleClient, OpenAiTransport};
use crate::ai::provider_resolver::OpenAiChatProvider;
use crate::ai::settings::AiProviderKind;

pub const AGENT_SYSTEM_PROMPT: &str = "你是 Ameya 的项目资料问答助手。只能基于用户提供的项目上下文回答；如果上下文不足以支持结论，明确说明不确定，并指出还需要哪些资料。回答要简洁、可执行，并在相关结论后引用上下文编号。";

pub const AGENT_TEMPERATURE: f32 = 0.2;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskReference {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub source: String,
    pub score: f32,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskRequest {
    pub project_id: String,
    pub question: String,
    pub references: Vec<AgentAskReference>,
}

#[derive(Debug, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentAskResponse {
    pub answer: String,
    pub references: Vec<AgentAskReference>,
    pub provider_kind: AiProviderKind,
    pub model: String,
}

pub fn validate_question(question: &str) -> Result<&str, String> {
    let question = question.trim();
    if question.is_empty() {
        return Err("请输入问题".into());
    }
    Ok(question)
}

pub fn run_agent_ask<T: OpenAiTransport>(
    request: AgentAskRequest,
    provider: OpenAiChatProvider,
    client: &OpenAiCompatibleClient<T>,
) -> Result<AgentAskResponse, String> {
    let question = validate_question(&request.question)?;
    let references = request.references;
    let messages = build_agent_messages(&request.project_id, question, &references);
    let answer = client
        .chat(&provider.model, messages, AGENT_TEMPERATURE)
        .map_err(|error| error.message)?;

    Ok(AgentAskResponse {
        answer,
        references,
        provider_kind: AiProviderKind::OpenAiCompatible,
        model: provider.model,
    })
}

pub fn build_agent_messages(
    project_id: &str,
    question: &str,
    references: &[AgentAskReference],
) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "system".into(),
            content: AGENT_SYSTEM_PROMPT.into(),
        },
        ChatMessage {
            role: "user".into(),
            content: build_agent_user_prompt(project_id, question, references),
        },
    ]
}

pub fn build_agent_user_prompt(
    project_id: &str,
    question: &str,
    references: &[AgentAskReference],
) -> String {
    let mut prompt = format!("项目 ID：{project_id}\n\n问题：{question}\n\n项目上下文：");
    if references.is_empty() {
        prompt.push_str("\n无可用引用。请明确说明上下文不足。");
        return prompt;
    }

    for (index, reference) in references.iter().enumerate() {
        prompt.push_str(&format!(
            "\n[{}] {} {}（{}:{}，来源：{}，分数：{:.2}）\n{}",
            index + 1,
            reference.entity_type,
            reference.title,
            reference.entity_type,
            reference.entity_id,
            reference.source,
            reference.score,
            reference.snippet
        ));
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::{
        build_agent_messages, build_agent_user_prompt, run_agent_ask, validate_question,
        AgentAskReference, AgentAskRequest, AgentAskResponse, AGENT_SYSTEM_PROMPT,
        AGENT_TEMPERATURE,
    };
    use crate::ai::openai_compatible::{
        OpenAiCompatibleClient, OpenAiRequest, OpenAiResponse, OpenAiTransport, ProviderError,
    };
    use crate::ai::provider_resolver::OpenAiChatProvider;
    use crate::ai::settings::AiProviderKind;

    fn reference(overrides: impl FnOnce(&mut AgentAskReference)) -> AgentAskReference {
        let mut reference = AgentAskReference {
            entity_type: "entry".into(),
            entity_id: "entry_1".into(),
            title: "月光阔剑".into(),
            snippet: "潮汐能武器，首次用于围城战。".into(),
            source: "keyword".into(),
            score: 2.0,
        };
        overrides(&mut reference);
        reference
    }

    fn chat_provider() -> OpenAiChatProvider {
        OpenAiChatProvider {
            base_url: "https://llm.example/v1".into(),
            model: "story-chat".into(),
            api_key: "sk-test".into(),
        }
    }

    struct StubTransport;

    impl OpenAiTransport for StubTransport {
        fn post_json(&self, _request: OpenAiRequest) -> Result<OpenAiResponse, ProviderError> {
            Ok(OpenAiResponse {
                status: 200,
                body: r#"{"choices":[{"message":{"role":"assistant","content":"ok answer"}}]}"#
                    .into(),
            })
        }
    }

    #[test]
    fn ai_agent_prompt_includes_question_and_numbered_references() {
        let prompt = build_agent_user_prompt(
            "project_1",
            "月光阔剑有什么风险？",
            &[
                reference(|_| {}),
                reference(|item| {
                    item.entity_type = "event".into();
                    item.entity_id = "event_1".into();
                    item.title = "围城战".into();
                    item.snippet = "围城战暴露了能源补给问题。".into();
                    item.source = "semantic".into();
                    item.score = 0.78;
                }),
            ],
        );

        assert!(prompt.contains("项目 ID：project_1"));
        assert!(prompt.contains("问题：月光阔剑有什么风险？"));
        assert!(prompt.contains("[1] entry 月光阔剑"));
        assert!(prompt.contains("潮汐能武器"));
        assert!(prompt.contains("[2] event 围城战"));
        assert!(prompt.contains("来源：semantic"));
        assert!(prompt.contains("分数：0.78"));
    }

    #[test]
    fn ai_agent_prompt_marks_missing_context() {
        let prompt = build_agent_user_prompt("project_1", "谁掌握能源？", &[]);

        assert!(prompt.contains("无可用引用"));
        assert!(prompt.contains("上下文不足"));
    }

    #[test]
    fn ai_agent_messages_pair_system_prompt_with_user_prompt() {
        let messages = build_agent_messages("project_1", "谁掌握能源？", &[reference(|_| {})]);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, AGENT_SYSTEM_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("谁掌握能源？"));
    }

    #[test]
    fn validate_question_rejects_blank_input() {
        assert_eq!(validate_question("  "), Err("请输入问题".into()));
        assert_eq!(validate_question("  问题 "), Ok("问题"));
    }

    #[test]
    fn run_agent_ask_rejects_empty_question_before_calling_provider() {
        let request = AgentAskRequest {
            project_id: "project_1".into(),
            question: "   ".into(),
            references: vec![reference(|_| {})],
        };
        let client = OpenAiCompatibleClient::new(
            "https://llm.example/v1".into(),
            "sk-test".into(),
            StubTransport,
        );

        let error = run_agent_ask(request, chat_provider(), &client)
            .expect_err("empty question should fail");

        assert_eq!(error, "请输入问题");
    }

    #[test]
    fn run_agent_ask_returns_answer_with_references_and_model() {
        let request = AgentAskRequest {
            project_id: "project_1".into(),
            question: "月光阔剑有什么风险？".into(),
            references: vec![reference(|_| {})],
        };
        let provider = chat_provider();
        let client = OpenAiCompatibleClient::new(
            provider.base_url.clone(),
            provider.api_key.clone(),
            StubTransport,
        );

        let response = run_agent_ask(request, provider, &client).expect("agent ask succeeds");

        assert_eq!(
            response,
            AgentAskResponse {
                answer: "ok answer".into(),
                references: vec![reference(|_| {})],
                provider_kind: AiProviderKind::OpenAiCompatible,
                model: "story-chat".into(),
            }
        );
        // AGENT_TEMPERATURE must stay at the Tauri-contract value.
        assert_eq!(AGENT_TEMPERATURE, 0.2);
    }
}
