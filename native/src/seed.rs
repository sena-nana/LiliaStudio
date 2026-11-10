//! 离线快照的确定性数据注入。
//!
//! `AMEYA_SNAPSHOT_SEED=1` 时由 `AmeyaApp::initialize` 调用：向 store 写入
//! 一套固定的示例数据，让真实视图在 headless 快照进程里渲染出完整布局
//! （项目列表、资料库行、编辑器表单、状态卡）。不落库、不触发命令。

use crate::state::{
    AgentAnswer, AgentReference, AiSettingsDraft, AppState, Collection, IndexPhase, ObjectRef,
    StatusLine, StatusTone,
};
use ameya_core::ai::settings::{AiProviderKind, AiProviderSettingsView};
use ameya_core::domain::axiom::Axiom;
use ameya_core::domain::character::Character;
use ameya_core::domain::character_trait_delta::CharacterTraitDeltaRecord;
use ameya_core::domain::entry::Entry;
use ameya_core::domain::event::Event;
use ameya_core::domain::project::Project;
use ameya_core::domain::relation::{EntityRef, Relation};
use ameya_core::logic::conflict::LogicConflict;
use ameya_core::logic::repair::RepairSuggestion;
use ameya_core::services::character_growth::{
    CharacterGrowthWorkspaceSnapshot, CharacterTraitState,
};
use ameya_core::services::jobs::AiJob;
use ameya_core::services::jobs::AiJobLog;
use ameya_core::services::prompts::{PromptTemplate, PromptTemplateVariable};
use ameya_core::services::rag::RagIndexStatus;
use ameya_core::services::search::{SearchResult, SemanticSearchResponse, SemanticSearchStatus};
use ameya_core::services::simulation::{SimulationPhase, SimulationReport};
use nana_ui::runtime::view::{Store, StorePath};

pub fn seed_snapshot_store(store: &Store<AppState>) {
    let ts = "2026-11-09T00:00:00Z";
    store.update(|state| {
        state.status = StatusLine {
            text: "就绪 · 本地数据目录已连接".into(),
            tone: StatusTone::Ok,
        };

        let project = Project {
            id: "p-sample-1".into(),
            name: "示例项目".into(),
            description: "用于离线快照的确定性示例数据".into(),
            created_at: ts.into(),
            updated_at: ts.into(),
            archived_at: None,
        };
        state.projects.list = vec![project];
        state.projects.active_id = Some("p-sample-1".into());
        state.projects.create_name = "未命名宇宙".into();
        state.projects.create_description = "世界观设定的简短描述".into();

        state.library.entries = vec![
            Entry {
                id: "entry-1".into(),
                project_id: "p-sample-1".into(),
                entry_type: "概念".into(),
                title: "月光金属".into(),
                summary: "第三纪北方出产的发光矿石".into(),
                body: r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.into(),
                tags: vec!["矿物".into(), "北方".into()],
                status: "draft".into(),
                created_at: ts.into(),
                updated_at: ts.into(),
                deleted_at: None,
            },
            Entry {
                id: "entry-2".into(),
                project_id: "p-sample-1".into(),
                entry_type: "地点".into(),
                title: "北方城墙".into(),
                summary: "抵御冻原风暴的古城墙".into(),
                body: r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.into(),
                tags: vec!["建筑".into()],
                status: "published".into(),
                created_at: ts.into(),
                updated_at: ts.into(),
                deleted_at: None,
            },
        ];
        state.library.characters = vec![Character {
            id: "char-1".into(),
            project_id: "p-sample-1".into(),
            name: "阿离".into(),
            aliases: vec!["北境守夜人".into()],
            summary: "守夜人队伍的年轻队长".into(),
            appearance: "银灰色斗篷，左颊有旧疤".into(),
            goals: "查明月光金属失踪的真相".into(),
            motivations: "守护北方城墙的居民".into(),
            fears: "冻原风暴再次南下".into(),
            faction: "守夜人".into(),
            tags: vec!["主角".into()],
            created_at: ts.into(),
            updated_at: ts.into(),
            deleted_at: None,
        }];
        state.library.events = vec![Event {
            id: "event-1".into(),
            project_id: "p-sample-1".into(),
            title: "饥荒之夜".into(),
            description: "北方粮食短缺引发骚动".into(),
            time_label: "第三纪 214 年冬".into(),
            sort_key: 214,
            start_label: "第三纪 214 年".into(),
            end_label: "第三纪 215 年".into(),
            location: "北方城墙".into(),
            importance: 4,
            outcome: "城墙守军开始配给制".into(),
            tags: vec!["危机".into()],
            created_at: ts.into(),
            updated_at: ts.into(),
            deleted_at: None,
        }];
        state.library.axioms = vec![
            Axiom {
                id: "axiom-1".into(),
                project_id: "p-sample-1".into(),
                subject: "月光金属".into(),
                predicate: "state".into(),
                object: "solid".into(),
                scope_time: "第三纪".into(),
                scope_location: "北方".into(),
                certainty: 1.0,
                source_entity_type: Some("entry".into()),
                source_entity_id: Some("entry-1".into()),
                natural_language: "月光金属在第三纪的北方呈固态".into(),
                tags: vec![],
                created_at: ts.into(),
                updated_at: ts.into(),
                deleted_at: None,
            },
            Axiom {
                id: "axiom-2".into(),
                project_id: "p-sample-1".into(),
                subject: "月光金属".into(),
                predicate: "state".into(),
                object: "liquid".into(),
                scope_time: "第三纪".into(),
                scope_location: "北方".into(),
                certainty: 0.9,
                source_entity_type: None,
                source_entity_id: None,
                natural_language: "部分记录称月光金属呈液态".into(),
                tags: vec![],
                created_at: ts.into(),
                updated_at: ts.into(),
                deleted_at: None,
            },
        ];
        state.library.relations = vec![Relation {
            id: "rel-1".into(),
            project_id: "p-sample-1".into(),
            source: EntityRef {
                entity_type: "entry".into(),
                entity_id: "entry-1".into(),
            },
            target: EntityRef {
                entity_type: "character".into(),
                entity_id: "char-1".into(),
            },
            relation_type: "产出地守护".into(),
            description: "阿离守护月光金属的矿脉".into(),
            confidence: 0.9,
            directed: true,
            created_at: ts.into(),
            updated_at: ts.into(),
            deleted_at: None,
        }];
        state.library.collection = Some(Collection::Entry);
        state.library.selected = Some(ObjectRef {
            kind: Collection::Entry,
            id: "entry-1".into(),
        });

        state.search.query = "月光金属".into();
        state.search.keyword_results = vec![
            SearchResult {
                entity_type: "entry".into(),
                entity_id: "entry-1".into(),
                title: "月光金属".into(),
                snippet: "第三纪北方出产的发光矿石".into(),
                score: 2.0,
            },
            SearchResult {
                entity_type: "character".into(),
                entity_id: "char-1".into(),
                title: "阿离".into(),
                snippet: "守夜人队伍的年轻队长".into(),
                score: 1.5,
            },
        ];
        state.search.semantic = Some(SemanticSearchResponse {
            status: SemanticSearchStatus::Ready,
            message: String::new(),
            model: "text-embedding-3-small".into(),
            items: vec![],
        });

        state.agent.question = "这个世界的主食是什么？".into();
        state.agent.answer = Some(AgentAnswer {
            answer: "北方以冻麦与岩鱼为主食，饥荒时期改以地衣汤度日。".into(),
            provider_label: "OpenAI 兼容".into(),
            model: "gpt-4o-mini".into(),
            references: vec![AgentReference {
                title: "月光金属".into(),
                entity_type: "entry".into(),
                entity_id: "entry-1".into(),
                snippet: "第三纪北方出产的发光矿石".into(),
                source: "keyword".into(),
                score: 2.0,
            }],
        });

        state.jobs.list = vec![AiJob {
            id: "job-1".into(),
            project_id: Some("p-sample-1".into()),
            provider_kind: "openAiCompatible".into(),
            job_type: "agentChat".into(),
            status: "queued".into(),
            input_summary: "询问智能体：这个世界的主食是什么？".into(),
            output_text: String::new(),
            error_message: None,
            started_at: None,
            finished_at: None,
            cancel_requested_at: None,
            retry_of_job_id: None,
            created_at: ts.into(),
            updated_at: ts.into(),
        }];
        state.jobs.current = state.jobs.list.first().cloned();
        state.jobs.logs = vec![AiJobLog {
            id: "log-1".into(),
            job_id: "job-1".into(),
            level: "info".into(),
            message: "任务已排队，等待执行".into(),
            created_at: ts.into(),
        }];

        state.prompts.templates = vec![PromptTemplate {
            id: "prompt-1".into(),
            name: "角色对白润色".into(),
            purpose: "按角色口吻改写对白".into(),
            template: "请以 {{主角}} 的口吻改写以下对白：".into(),
            built_in: true,
            variables: vec![PromptTemplateVariable {
                name: "主角".into(),
                description: "出场角色名".into(),
                example: "阿离".into(),
            }],
            created_at: ts.into(),
            updated_at: ts.into(),
        }];
        state.prompts.selected_id = Some("prompt-1".into());
        state.prompts.fill_draft_from_selected();
        state.prompts.variable_values = vec![("主角".to_string(), "阿离".to_string())];

        state.indexing.status = Some(RagIndexStatus {
            chunk_count: 12,
            embedding_count: 12,
            missing_embedding_count: 0,
            stale_embedding_count: 0,
            model: "text-embedding-3-small".into(),
            status: "ready".into(),
            message: String::new(),
        });
        state.indexing.phase = IndexPhase::Ready;

        let mut values = std::collections::BTreeMap::new();
        values.insert("勇气".to_string(), 0.5);
        values.insert("谨慎".to_string(), -0.25);
        state.growth.workspace = Some(CharacterGrowthWorkspaceSnapshot {
            records: vec![CharacterTraitDeltaRecord {
                id: "growth-1".into(),
                project_id: "p-sample-1".into(),
                character_id: "char-1".into(),
                source_event_id: "event-1".into(),
                trait_name: "勇气".into(),
                delta: 0.5,
                reason: "饥荒之夜挺身而出".into(),
                created_at: ts.into(),
                updated_at: ts.into(),
                deleted_at: None,
            }],
            states: [(
                "char-1".to_string(),
                CharacterTraitState {
                    values,
                    sources: vec![],
                },
            )]
            .into_iter()
            .collect(),
        });
        state.growth.selected_character = Some("char-1".into());
        state.growth.source_event = Some("event-1".into());
        state.growth.trait_name = "勇气".into();
        state.growth.delta_text = "+0.5".into();
        state.growth.reason = "饥荒之夜挺身而出".into();

        state.simulation.report = Some(SimulationReport {
            project_id: "p-sample-1".into(),
            scenario: "如果北方发生饥荒".into(),
            phases: vec![SimulationPhase {
                label: "初期".into(),
                summary: "粮价上涨，边境贸易紧张".into(),
            }],
            risks: vec!["城墙守军士气下降".into()],
            referenced_entities: vec!["粮食".into(), "北方城墙".into()],
        });

        state.audit.conflicts = vec![LogicConflict {
            conflict_type: "mutually_exclusive_axioms".into(),
            fact_ids: vec!["fact_1".into(), "fact_2".into()],
            message: "月光金属在第三纪·北方同时为固态与液态".into(),
        }];
        state.audit.repairs = vec![RepairSuggestion {
            title: "限定作用域".into(),
            description: "将液态记录限定到南方矿脉".into(),
            impact: "消除互斥冲突，保留两条记录".into(),
        }];

        state.backup.import_text = r#"{"version":1,"project":{}}"#.into();

        state.ai_settings.views = vec![
            AiProviderSettingsView {
                kind: AiProviderKind::OpenAiCompatible,
                base_url: Some("https://api.example.com/v1".into()),
                api_key_preview: Some("sk-***abcd".into()),
                has_api_key: true,
                chat_model: Some("gpt-4o-mini".into()),
                embedding_model: Some("text-embedding-3-small".into()),
                command_template: None,
                enabled: true,
            },
            AiProviderSettingsView {
                kind: AiProviderKind::CodexCli,
                base_url: None,
                api_key_preview: None,
                has_api_key: false,
                chat_model: None,
                embedding_model: None,
                command_template: Some(
                    "codex exec --cd \"{workspace}\" --ask-for-approval never \"{prompt}\"".into(),
                ),
                enabled: false,
            },
            AiProviderSettingsView {
                kind: AiProviderKind::ClaudeCli,
                base_url: None,
                api_key_preview: None,
                has_api_key: false,
                chat_model: None,
                embedding_model: None,
                command_template: Some("claude -p \"{prompt}\" --output-format json".into()),
                enabled: false,
            },
        ];
        state.ai_settings.loaded = true;
        state.ai_settings.drafts = state
            .ai_settings
            .views
            .iter()
            .map(draft_from_view)
            .collect();

        state.diagnostics.app_version = Some(env!("CARGO_PKG_VERSION").into());
        state.diagnostics.platform = Some("windows".into());
        state.diagnostics.database = Some("sqlite".into());
    });
}

fn draft_from_view(view: &AiProviderSettingsView) -> AiSettingsDraft {
    let (key, label) = match view.kind {
        AiProviderKind::OpenAiCompatible => ("openAiCompatible", "OpenAI 兼容"),
        AiProviderKind::CodexCli => ("codexCli", "Codex CLI"),
        AiProviderKind::ClaudeCli => ("claudeCli", "Claude CLI"),
    };
    AiSettingsDraft {
        kind: key.into(),
        label: label.into(),
        base_url: view.base_url.clone().unwrap_or_default(),
        api_key: String::new(),
        clear_api_key: false,
        chat_model: view.chat_model.clone().unwrap_or_default(),
        embedding_model: view.embedding_model.clone().unwrap_or_default(),
        command_template: view.command_template.clone().unwrap_or_default(),
        enabled: view.enabled,
    }
}
