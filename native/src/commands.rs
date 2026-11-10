//! 命令层：AppMessage 数据请求 → NativeCommandHost 调用 → CommandOutcome。
//!
//! 载荷从 store 现值构建（缺上下文的命令在这里失败并给出中文提示），
//! 返回值统一经 serde 反序列化回核心类型，保持 camelCase 契约零漂移。
//! 本地 SQLite 命令在 update 内联执行；网络命令（AI/embedding/语义检索）
//! 由 app.rs 走 `run_task`，这里只提供纯函数体。

use ameya_core::services::{
    library::LibraryProjectSnapshot, relation_workspace::RelationNeighborhood,
};
use serde_json::json;

use crate::host::NativeCommandHost;
use crate::state::{AppState, CommandOutcome};

/// 从 store 现值构建命令载荷；缺少前置上下文时给出中文错误。
pub fn payload_for(state: &AppState, command: &str) -> Result<serde_json::Value, String> {
    let project_id = || {
        state
            .active_project_id()
            .map(str::to_string)
            .ok_or_else(|| "请先选择一个项目".to_string())
    };
    match command {
        "project_list"
        | "job_list"
        | "prompt_list_templates"
        | "health_check"
        | "diagnostics_summary"
        | "ai_default_providers"
        | "library_relation_type_presets" => Ok(json!({})),
        "project_create" => {
            let name = state.projects.create_name.trim();
            if name.is_empty() {
                return Err("项目名称不能为空".to_string());
            }
            Ok(
                json!({"draft": {"name": name, "description": state.projects.create_description.trim()}}),
            )
        }
        "library_project_snapshot"
        | "character_growth_workspace"
        | "archive_export_project"
        | "rag_index_status"
        | "rag_preview_context_pack" => Ok(json!({"projectId": project_id()?})),
        "search_entities" => Ok(json!({"filter": {
            "projectId": project_id()?,
            "query": state.search.query,
            "entityTypes": [],
        }})),
        "search_semantic" => Ok(json!({"request": {
            "projectId": project_id()?,
            "query": state.search.query,
            "limit": 8,
        }})),
        // 智能体提问：引用资料取当前关键词检索结果（前端驱动 RAG，与 Tauri 版一致）。
        "ai_agent_ask" => {
            let references: Vec<_> = state
                .search
                .keyword_results
                .iter()
                .take(8)
                .map(|result| {
                    json!({
                        "entityType": result.entity_type,
                        "entityId": result.entity_id,
                        "title": result.title,
                        "snippet": result.snippet,
                        "source": "keyword",
                        "score": result.score,
                    })
                })
                .collect();
            Ok(json!({"request": {
                "projectId": project_id()?,
                "question": state.agent.question,
                "references": references,
            }}))
        }
        // 智能体回答保存为报告词条（与资料库编辑器的 create 命令分离，载荷来源不同）。
        "agent_save_report" => {
            let Some(answer) = state.agent.answer.as_ref() else {
                return Err("当前没有可保存的回答".to_string());
            };
            let question = state.agent.question.trim();
            let title = if question.chars().count() > 30 {
                let prefix: String = question.chars().take(30).collect();
                format!("{prefix}…")
            } else if question.is_empty() {
                "智能体回答".to_string()
            } else {
                question.to_string()
            };
            let mut body = answer.answer.clone();
            if !answer.references.is_empty() {
                body.push_str("\n\n参考资料：\n");
                for (index, reference) in answer.references.iter().enumerate() {
                    body.push_str(&format!(
                        "[{}] {}（{}:{}，来源：{}，分数：{:.2}）\n",
                        index + 1,
                        reference.title,
                        reference.entity_type,
                        reference.entity_id,
                        reference.source,
                        reference.score
                    ));
                }
            }
            Ok(json!({"draft": {
                "projectId": project_id()?,
                "entryType": "report",
                "title": title,
                "summary": question,
                "body": body,
                "tags": ["agent", "report"],
                "status": "draft",
            }}))
        }
        // 智能体提问保存为后台任务记录。
        "job_create" => {
            let question = state.agent.question.trim();
            Ok(json!({"draft": {
                "projectId": project_id()?,
                "providerKind": "openAiCompatible",
                "jobType": "agentChat",
                "inputSummary": if question.is_empty() { "智能体问答" } else { question },
            }}))
        }
        "job_cancel" | "job_retry" | "job_logs" => Err("任务命令需要具体的任务 ID".to_string()),
        "logic_audit_facts" => Ok(json!({"facts": [
            {"id": "fact_1", "subject": "月光金属", "predicate": "state", "object": "solid",
             "scopeTime": "第三纪", "scopeLocation": "北方"},
            {"id": "fact_2", "subject": "月光金属", "predicate": "state", "object": "liquid",
             "scopeTime": "第三纪", "scopeLocation": "北方"},
        ]})),
        "logic_repair_suggestions" => {
            let Some(conflict) = state.audit.conflicts.first() else {
                return Err("当前没有冲突可生成修复建议".to_string());
            };
            Ok(json!({"conflict": conflict}))
        }
        "prompt_save_template" => Ok(json!({"draft": {
            "id": state.prompts.draft_id,
            "name": state.prompts.draft_name,
            "purpose": state.prompts.draft_purpose,
            "template": state.prompts.draft_template,
        }})),
        "prompt_preview_template" => Ok(json!({"request": {
            "template": state.prompts.draft_template,
            "values": state.prompts.variable_values.iter().map(|(name, value)| {
                json!({"name": name, "value": value})
            }).collect::<Vec<_>>(),
        }})),
        "prompt_reset_builtin_templates"
        | "ai_test_openai_provider"
        | "ai_test_codex_cli_provider"
        | "ai_test_claude_cli_provider" => Ok(json!({})),
        "ai_save_provider_settings" => Ok(json!({"drafts": state
            .ai_settings
            .drafts
            .iter()
            .map(|draft| {
                json!({
                    "kind": draft.kind,
                    "baseUrl": draft.base_url,
                    "apiKey": draft.api_key,
                    "clearApiKey": draft.clear_api_key,
                    "chatModel": draft.chat_model,
                    "embeddingModel": draft.embedding_model,
                    "commandTemplate": draft.command_template,
                    "enabled": draft.enabled,
                })
            })
            .collect::<Vec<_>>()})),
        "simulation_run" => Ok(json!({
            "projectId": project_id()?,
            "scenario": "如果北方发生饥荒",
            "referencedEntities": ["粮食", "北方城墙"],
        })),
        "archive_import_project" => {
            let text = state.backup.import_text.trim();
            if text.is_empty() {
                return Err("请先粘贴要导入的归档 JSON".to_string());
            }
            let archive: serde_json::Value = serde_json::from_str(text)
                .map_err(|error| format!("归档 JSON 解析失败：{error}"))?;
            Ok(json!({"archive": archive}))
        }
        "rag_index_embeddings" => Ok(json!({"projectId": project_id()?})),
        "library_relation_neighborhood" => {
            let Some(selected) = state.library.selected.as_ref() else {
                return Err("请先在资料库中选择一个对象".to_string());
            };
            Ok(json!({
                "projectId": project_id()?,
                "target": {"entityType": selected.kind.entity_key(), "entityId": selected.id},
                "depth": 1,
            }))
        }
        // —— 对象工作台：五类草稿（编辑器草稿 → 各自 Draft 契约）——
        "library_create_entry" | "library_update_entry" => {
            let editor = &state.library.editor;
            let draft = json!({
                "projectId": project_id()?,
                "entryType": editor.entry_type,
                "title": editor.title,
                "summary": editor.summary,
                // 摘要/正文草稿是纯文本，落库包成 TipTap 文档（与 core 语义一致）。
                "body": ameya_core::domain::entry_rich_text::normalize_entry_rich_text(&editor.body),
                "tags": split_tags(&editor.tags),
                "status": editor.status,
            });
            library_payload(state, command, "entry", draft)
        }
        "library_create_character" | "library_update_character" => {
            let editor = &state.library.editor;
            let draft = json!({
                "projectId": project_id()?,
                "name": editor.name,
                "aliases": split_tags(&editor.aliases),
                "summary": editor.summary,
                "appearance": editor.appearance,
                "goals": editor.goals,
                "motivations": editor.motivations,
                "fears": editor.fears,
                "faction": editor.faction,
                "tags": split_tags(&editor.tags),
            });
            library_payload(state, command, "character", draft)
        }
        "library_create_event" | "library_update_event" => {
            let editor = &state.library.editor;
            let draft = json!({
                "projectId": project_id()?,
                "title": editor.title,
                "description": editor.description,
                "timeLabel": editor.time_label,
                "sortKey": editor.sort_key.round() as i64,
                "startLabel": editor.start_label,
                "endLabel": editor.end_label,
                "location": editor.location,
                "importance": editor.importance.round() as i64,
                "outcome": editor.outcome,
                "tags": split_tags(&editor.tags),
            });
            library_payload(state, command, "event", draft)
        }
        "library_create_axiom" | "library_update_axiom" => {
            let editor = &state.library.editor;
            let draft = json!({
                "projectId": project_id()?,
                "subject": editor.subject,
                "predicate": editor.predicate,
                "object": editor.object,
                "scopeTime": editor.scope_time,
                "scopeLocation": editor.scope_location,
                "certainty": editor.certainty,
                "sourceEntityType": editor.source_entity_type,
                "sourceEntityId": editor.source_entity_id,
                "naturalLanguage": editor.natural_language,
                "tags": split_tags(&editor.tags),
            });
            library_payload(state, command, "axiom", draft)
        }
        "library_create_relation" | "library_update_relation" => {
            let editor = &state.library.editor;
            let source = parse_entity_ref(&editor.relation_source)?;
            let target = parse_entity_ref(&editor.relation_target)?;
            let draft = json!({
                "projectId": project_id()?,
                "source": {"entityType": source.0, "entityId": source.1},
                "target": {"entityType": target.0, "entityId": target.1},
                "relationType": editor.relation_type,
                "description": editor.summary,
                "confidence": editor.confidence,
                "directed": editor.directed,
            });
            library_payload(state, command, "relation", draft)
        }
        "library_delete_entry"
        | "library_delete_character"
        | "library_delete_event"
        | "library_delete_axiom"
        | "library_delete_relation" => {
            let Some(selected) = state.library.selected.as_ref() else {
                return Err("请先选择要删除的对象".to_string());
            };
            Ok(json!({"id": selected.id}))
        }
        // —— 角色成长 ——
        "character_growth_preview_trait_delta" => {
            let growth = &state.growth;
            let Some(workspace) = growth.workspace.as_ref() else {
                return Err("成长工作台尚未加载".to_string());
            };
            let Some(character_id) = growth.selected_character.as_ref() else {
                return Err("请先选择角色".to_string());
            };
            let Some(trait_state) = workspace.states.get(character_id) else {
                return Err("所选角色没有特质状态".to_string());
            };
            let delta = parse_trait_delta(&growth.delta_text)?;
            Ok(json!({
                "state": trait_state,
                "delta": {
                    "sourceEventId": growth.source_event,
                    "traitName": growth.trait_name,
                    "delta": delta,
                    "reason": growth.reason,
                },
            }))
        }
        "character_growth_create_record" => {
            let growth = &state.growth;
            let delta = parse_trait_delta(&growth.delta_text)?;
            Ok(json!({"draft": {
                "projectId": project_id()?,
                "characterId": growth.selected_character,
                "sourceEventId": growth.source_event,
                "traitName": growth.trait_name,
                "delta": delta,
                "reason": growth.reason,
            }}))
        }
        _ => Ok(json!({})),
    }
}

/// create/update 共用载荷：update 附带当前选中对象 ID。
fn library_payload(
    state: &AppState,
    command: &str,
    kind: &str,
    draft: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if command.starts_with("library_update_") {
        let Some(selected) = state.library.selected.as_ref() else {
            return Err("请先选择要编辑的对象".to_string());
        };
        if selected.kind.entity_key() != kind {
            return Err("编辑的对象类型与当前集合不一致".to_string());
        }
        Ok(json!({"id": selected.id, "draft": draft}))
    } else {
        Ok(json!({"draft": draft}))
    }
}

/// 「类型:ID」引用（关系来源/目标下拉的键）。
fn parse_entity_ref(value: &str) -> Result<(String, String), String> {
    let value = value.trim();
    let (kind, id) = value
        .split_once(':')
        .ok_or_else(|| "关系端点格式应为「类型:ID」".to_string())?;
    if kind.is_empty() || id.is_empty() {
        return Err("关系端点格式应为「类型:ID」".to_string());
    }
    Ok((kind.to_string(), id.to_string()))
}

/// 标签/别名拆分（Tauri 契约：中英文逗号）。
fn split_tags(value: &str) -> Vec<String> {
    value
        .split([',', '，'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// 特质变化解析：+0.5 / -0.5 / 0.5。
fn parse_trait_delta(value: &str) -> Result<f32, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("请输入特质变化（示例：+0.5 / -0.5）".to_string());
    }
    let magnitude = value.trim_start_matches(['+', '-']);
    let parsed: f32 = magnitude
        .parse()
        .map_err(|_| "特质变化格式无效（示例：+0.5 / -0.5）".to_string())?;
    if value.starts_with('-') {
        Ok(-parsed)
    } else {
        Ok(parsed)
    }
}

/// 携带任务 ID 的命令载荷（任务列表按钮直接带 ID，不经过 store 选择态）。
pub fn job_payload(job_id: &str) -> serde_json::Value {
    json!({"jobId": job_id})
}

/// 把 host 返回的 JSON 转成类型化结果。未知命令按原值透传。
pub fn parse_outcome(command: &str, value: serde_json::Value) -> CommandOutcome {
    match command {
        "project_list" => CommandOutcome::Projects(from_value(value)),
        "library_project_snapshot" => {
            CommandOutcome::LibrarySnapshot(Box::new(from_value::<LibraryProjectSnapshot>(value)))
        }
        "job_list" => CommandOutcome::Jobs(from_value(value)),
        "job_current" => CommandOutcome::CurrentJob(from_value(value)),
        "job_logs" => CommandOutcome::JobLogs(from_value(value)),
        "prompt_list_templates" => CommandOutcome::Prompts(from_value(value)),
        "prompt_preview_template" => CommandOutcome::PromptPreview(from_value(value)),
        "library_relation_type_presets" => CommandOutcome::RelationPresets(from_value(value)),
        "library_relation_neighborhood" => {
            CommandOutcome::Neighborhood(Box::new(from_value::<RelationNeighborhood>(value)))
        }
        "ai_load_provider_settings" => CommandOutcome::ProviderSettings(from_value(value)),
        "diagnostics_summary" => CommandOutcome::Value(value),
        "rag_index_status" => CommandOutcome::RagStatus(from_value(value)),
        "search_entities" => CommandOutcome::SearchResults(from_value(value)),
        "search_semantic" => CommandOutcome::SemanticSearch(from_value(value)),
        "character_growth_workspace" => {
            CommandOutcome::GrowthWorkspace(Box::new(from_value(value)))
        }
        "simulation_run" => CommandOutcome::SimulationReport(from_value(value)),
        "logic_audit_facts" => CommandOutcome::AuditConflicts(from_value(value), Vec::new()),
        "logic_repair_suggestions" => CommandOutcome::AuditRepairs(from_value(value)),
        "vector_preview_chunks" => CommandOutcome::Value(value),
        _ => CommandOutcome::Value(value),
    }
}

fn from_value<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).expect("host 返回值与核心类型契约一致")
}

/// 内联（SQLite/本地）命令执行：构建载荷 → invoke → 类型化。
pub fn run_inline(
    host: &NativeCommandHost,
    state: &AppState,
    command: &str,
) -> Result<CommandOutcome, String> {
    let payload = payload_for(state, command)?;
    let value = host.invoke(command, payload)?;
    Ok(parse_outcome(command, value))
}

/// 网络命令的纯任务体：在后台线程执行，失败被捕获为 Err 字符串。
pub fn run_network(
    host: &std::sync::Arc<NativeCommandHost>,
    state_snapshot: AppState,
    command: &str,
) -> Result<CommandOutcome, String> {
    let payload = payload_for(&state_snapshot, command)?;
    let value = host.invoke(command, payload)?;
    Ok(parse_outcome(command, value))
}

/// 需要走后台线程的命令清单（HTTP/CLI 子进程）。
pub fn is_network_command(command: &str) -> bool {
    matches!(
        command,
        "ai_agent_ask"
            | "ai_test_openai_provider"
            | "ai_test_codex_cli_provider"
            | "ai_test_claude_cli_provider"
            | "rag_index_embeddings"
            | "search_semantic"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ameya_core::domain::{entry::Entry, event::Event, project::Project};
    use ameya_core::services::jobs::{AiJob, AiJobLog};
    use ameya_core::services::prompts::{PromptTemplate, PromptTemplatePreview};
    use ameya_core::services::rag::RagIndexStatus;
    use ameya_core::services::search::{SearchResult, SemanticSearchResponse};

    #[test]
    fn project_scoped_commands_require_selection() {
        let state = AppState::default();
        assert_eq!(
            payload_for(&state, "library_project_snapshot"),
            Err("请先选择一个项目".to_string())
        );
    }

    #[test]
    fn create_payload_carries_draft() {
        let mut state = AppState::default();
        state.projects.create_name = "星辰之书".into();
        state.projects.create_description = "西方奇幻".into();
        let payload = payload_for(&state, "project_create").unwrap();
        assert_eq!(payload["draft"]["name"], "星辰之书");
        assert_eq!(payload["draft"]["description"], "西方奇幻");
    }

    #[test]
    fn network_command_split_is_stable() {
        assert!(is_network_command("ai_agent_ask"));
        assert!(is_network_command("rag_index_embeddings"));
        assert!(!is_network_command("project_list"));
        assert!(!is_network_command("rag_index_status"));
    }

    #[test]
    fn outcome_round_trips_projects() {
        let project = Project {
            id: "p1".into(),
            name: "项目一".into(),
            description: String::new(),
            archived_at: None,
            created_at: "2026-11-10T00:00:00Z".into(),
            updated_at: "2026-11-10T00:00:00Z".into(),
        };
        let value = serde_json::to_value(vec![project]).unwrap();
        match parse_outcome("project_list", value) {
            CommandOutcome::Projects(projects) => assert_eq!(projects.len(), 1),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }

    #[test]
    fn entry_and_event_types_still_parse() {
        // 契约哨兵：资料库核心类型保持可反序列化。
        let entry: Entry = serde_json::from_value(json!({
            "id": "e1", "projectId": "p1", "entryType": "note",
            "title": "词条", "summary": "", "body": "", "tags": [],
            "status": "draft", "createdAt": "2026-11-10T00:00:00Z",
            "updatedAt": "2026-11-10T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(entry.title, "词条");
        let event: Event = serde_json::from_value(json!({
            "id": "v1", "projectId": "p1", "title": "事件", "description": "",
            "timeLabel": "第三纪", "sortKey": 0, "startLabel": "", "endLabel": "",
            "location": "", "importance": 1, "outcome": "", "tags": [],
            "createdAt": "2026-11-10T00:00:00Z",
            "updatedAt": "2026-11-10T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(event.title, "事件");
    }

    #[test]
    fn job_and_prompt_outcomes_parse() {
        let job: AiJob = serde_json::from_value(json!({
            "id": "j1", "projectId": "p1", "providerKind": "codexCli",
            "jobType": "agentChat", "status": "queued", "inputSummary": "",
            "outputText": "", "createdAt": "2026-11-10T00:00:00Z",
            "updatedAt": "2026-11-10T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(job.id, "j1");
        let log: AiJobLog = serde_json::from_value(json!({
            "id": "l1", "jobId": "j1", "level": "info", "message": "开始",
            "createdAt": "2026-11-10T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(log.message, "开始");
        let template: PromptTemplate = serde_json::from_value(json!({
            "id": "t1", "name": "审计", "purpose": "logicAudit",
            "template": "文本", "builtIn": true, "variables": [],
            "createdAt": "2026-11-10T00:00:00Z", "updatedAt": "2026-11-10T00:00:00Z"
        }))
        .unwrap();
        assert!(template.built_in);
        let preview: PromptTemplatePreview = serde_json::from_value(json!({
            "prompt": "结果", "missingVariables": []
        }))
        .unwrap();
        assert_eq!(preview.prompt, "结果");
        let status: RagIndexStatus = serde_json::from_value(json!({
            "chunkCount": 3, "embeddingCount": 2, "missingEmbeddingCount": 1,
            "staleEmbeddingCount": 0, "model": "m", "status": "ready", "message": ""
        }))
        .unwrap();
        assert_eq!(status.chunk_count, 3);
        let _neighborhood: RelationNeighborhood =
            serde_json::from_value(json!({"center": {"entityType": "character", "entityId": "c1", "title": "主角", "subtitle": "", "summary": ""}, "nodes": [], "edges": [], "suggestions": [], "missing": [], "relationCount": 0})).unwrap();
        let _search: SearchResult = serde_json::from_value(json!({
            "entityType": "entry", "entityId": "e1", "title": "t",
            "snippet": "", "source": "keyword", "score": 1.0
        }))
        .unwrap();
        let _semantic: SemanticSearchResponse = serde_json::from_value(
            json!({"status": "degraded", "message": "m", "model": "", "items": []}),
        )
        .unwrap();
    }
}
