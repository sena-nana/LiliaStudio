//! AmeyaApp：ApplicationState 适配层。持有响应式 store、命令 host 与
//! 命令面板引用；`update` 处理全部 AppMessage（导航 / 数据加载 / 命令回填），
//! `input_event` 拦截 Ctrl+K 开关命令面板。

use std::sync::Arc;
use std::time::Instant;

use nana_ui::runtime::view::EntityRef;
use nana_ui::runtime::view::{entity_ref, store, Store, StorePath};
use nana_ui::runtime::{
    BuiltinComponents, CommandPalette, DesktopShell, Entity, FrameworkError, NodeKind, OverlayHost,
    StableNodeId, Task, Workspace,
};
use nana_ui::{
    builtin_theme_arc, ApplicationState, ApplicationWindow, CompiledTheme, NanaApplication,
    RegionId, RuntimeProgramContext, RuntimeProgramUpdate, ThemeAppearance, WorkspaceMutation,
};
use nana_ui_platform::host::WindowCommand;
use nana_ui_platform::{InputPayload, KeyState, PointerPhase, WindowId};

use crate::commands;
use crate::host::NativeCommandHost;
use crate::shell;
use crate::state::{
    AgentStateStoreFields, AiSettingsStateStoreFields, AppMessage, AppState, AppStateStoreFields,
    AuditStateStoreFields, BackupStateStoreFields, BusyOp, Collection, CommandOutcome,
    DiagnosticsStateStoreFields, EditorField, EditorNumberField, GraphStateStoreFields,
    GrowthStateStoreFields, IndexingStateStoreFields, JobsStateStoreFields,
    LibraryStateStoreFields, ProjectsStateStoreFields, PromptsStateStoreFields,
    SearchStateStoreFields, SimulationStateStoreFields, StatusLine, StatusTone, ThemeChoice,
};
use crate::Route;

/// 命令面板悬停关闭的兜底提示时长。
const STATUS_OK: &str = "就绪";
/// 侧栏拖拽手柄双击判定窗口（与 Tauri 双击重置一致的手感阈值）。
const SIDEBAR_DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(500);

pub struct AmeyaApp {
    store: Store<AppState>,
    host: Option<Arc<NativeCommandHost>>,
    palette_host: EntityRef<OverlayHost>,
    palette: EntityRef<CommandPalette>,
    shell: EntityRef<DesktopShell>,
    /// 持久化的侧栏宽度（build 时注入工作区布局）。
    initial_sidebar_width: Option<f32>,
    /// 最近一次侧栏拖拽手柄按下（节点 + 时刻），用于双击重置判定。
    last_resize_click: Option<(StableNodeId, Instant)>,
}

impl AmeyaApp {
    fn set_status(&self, text: impl Into<String>, tone: StatusTone) {
        self.store.status().set(StatusLine {
            text: text.into(),
            tone,
        });
    }

    fn push_busy(&self, tag: &'static str, label: &'static str) {
        self.store.busy().update(|ops| {
            if !ops.iter().any(|op| op.tag == tag) {
                ops.push(BusyOp {
                    tag,
                    label: label.to_string(),
                });
            }
        });
    }

    fn pop_busy(&self, tag: &'static str) {
        self.store
            .busy()
            .update(|ops| ops.retain(|op| op.tag != tag));
    }

    /// 命令分发入口：本地 SQLite 内联执行，网络类走后台线程。
    pub fn dispatch_command(
        &mut self,
        command: &'static str,
        label: &'static str,
        context: &RuntimeProgramContext<AppMessage>,
    ) -> RuntimeProgramUpdate {
        if commands::is_network_command(command) {
            self.spawn_network_command(command, label, context)
        } else {
            self.run_local_command(command, label, context)
        }
    }

    /// 数据加载类命令：本地 SQLite，内联执行后立即回填。
    fn run_local_command(
        &mut self,
        command: &'static str,
        label: &'static str,
        context: &RuntimeProgramContext<AppMessage>,
    ) -> RuntimeProgramUpdate {
        let Some(host) = self.host.clone() else {
            self.set_status("本地数据目录不可用，命令未执行", StatusTone::Error);
            return RuntimeProgramUpdate::redraw_all();
        };
        self.push_busy(command, label);
        let snapshot = self.snapshot_state();
        let result = commands::run_inline(&host, &snapshot, command);
        let message = AppMessage::CommandDone(Box::new(crate::state::CommandDone {
            tag: command,
            outcome: result,
        }));
        let _ = context.dispatch(message);
        RuntimeProgramUpdate::redraw_all()
    }

    /// 网络类命令：后台线程执行，完成后以 CommandDone 消息回填。
    fn spawn_network_command(
        &mut self,
        command: &'static str,
        label: &'static str,
        context: &RuntimeProgramContext<AppMessage>,
    ) -> RuntimeProgramUpdate {
        let Some(host) = self.host.clone() else {
            self.set_status("本地数据目录不可用，命令未执行", StatusTone::Error);
            return RuntimeProgramUpdate::redraw_all();
        };
        self.push_busy(command, label);
        let snapshot = self.snapshot_state();
        let task = Task::new(async move {
            let outcome = commands::run_network(&host, snapshot, command);
            AppMessage::CommandDone(Box::new(crate::state::CommandDone {
                tag: command,
                outcome,
            }))
        });
        if let Err(error) = context.run_task(task) {
            self.pop_busy(command);
            self.set_status(format!("无法启动后台任务：{error}"), StatusTone::Error);
        }
        RuntimeProgramUpdate::redraw_all()
    }

    fn snapshot_state(&self) -> AppState {
        self.store.with(|state| state.clone())
    }

    /// Ctrl+K / 面板按钮共用的开关逻辑。
    fn toggle_palette(
        &mut self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
    ) -> Result<RuntimeProgramUpdate, FrameworkError> {
        let opening = !self.store.palette_open().get();
        self.store.palette_open().set(opening);
        if let Some(window) = windows.get_mut(&id) {
            let context = window.document.context_mut();
            if opening {
                if let (Some(host), Some(palette)) = (self.palette_host.get(), self.palette.get()) {
                    context.activate_overlay(host, palette)?;
                }
            } else if let Some(host) = self.palette_host.get() {
                let _ = context.dismiss_overlay(host);
            }
        }
        Ok(RuntimeProgramUpdate::redraw_all())
    }

    /// 解析 shell 挂载后的 Workspace 实体（侧栏宽度/折叠的真实权威）。
    fn workspace_entity_in(&self, window: &mut ApplicationWindow) -> Option<Entity<Workspace>> {
        let context = window.document.context_mut();
        let shell = self.shell.get()?;
        let workspace = context.read(shell, |shell| shell.workspace).ok()??;
        Some(Entity::<Workspace>::from_stable_id(workspace))
    }

    fn workspace_entity(
        &self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
    ) -> Option<Entity<Workspace>> {
        self.workspace_entity_in(windows.get_mut(&id)?)
    }

    /// 工作区突变时钟：WorkspaceController 语义的替代（进程内单调即可，
    /// 只驱动折叠/尺寸过渡动画的时间轴）。
    fn workspace_clock() -> std::time::Duration {
        Instant::now().elapsed()
    }

    /// 切换侧栏（Resources 区）折叠；返回切换后的折叠真值供持久化。
    fn toggle_sidebar_region(
        &mut self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
    ) -> Option<bool> {
        let entity = self.workspace_entity(id, windows)?;
        let window = windows.get_mut(&id)?;
        let context = window.document.context_mut();
        context
            .update_component(entity, |workspace, _| {
                workspace.model.update(
                    WorkspaceMutation::ToggleRegion(RegionId::Resources),
                    Self::workspace_clock(),
                );
            })
            .ok()?;
        let collapsed = context
            .read(entity, |workspace| {
                workspace
                    .model
                    .layout()
                    .region(&RegionId::Resources)
                    .map(|region| region.collapsed_value())
            })
            .ok()??;
        Some(collapsed)
    }

    /// 双击拖拽手柄：侧栏宽度回到布局默认（240，Tauri 契约）并立即持久化。
    fn reset_sidebar_region(
        &mut self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
    ) {
        if let Some(entity) = self.workspace_entity(id, windows) {
            let window = windows.get_mut(&id);
            if let Some(window) = window {
                let context = window.document.context_mut();
                let _ = context.update_component(entity, |workspace, _| {
                    workspace.model.update(
                        WorkspaceMutation::ResetRegionSize(RegionId::Resources),
                        Self::workspace_clock(),
                    );
                });
            }
        }
    }

    /// 读当前侧栏宽度并写入 kv（拖拽松手 / 重置 / 关窗时调用）。
    fn persist_sidebar_width(
        &self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
        context: &RuntimeProgramContext<AppMessage>,
    ) {
        let Some(entity) = self.workspace_entity(id, windows) else {
            return;
        };
        let extent = windows.get_mut(&id).and_then(|window| {
            window
                .document
                .context_mut()
                .read(entity, |workspace| {
                    workspace.model.region_extent(&RegionId::Resources)
                })
                .ok()
        });
        if let Some(extent) = extent {
            let _ = context
                .store()
                .set("ameya.sidebarWidth", extent.to_string());
        }
    }

    /// CommandDone 回填：busy 出栈 + 按类型写入 store。
    fn apply_outcome(&mut self, tag: &'static str, outcome: Result<CommandOutcome, String>) {
        self.pop_busy(tag);
        match outcome {
            Ok(outcome) => {
                let message = self.write_outcome(tag, outcome);
                self.set_status(message, StatusTone::Neutral);
            }
            Err(error) => {
                // 问答链路失败的兜底：复位提问态并回显错误。
                if matches!(tag, "search_entities" | "ai_agent_ask") {
                    let agent = self.store.agent();
                    if agent.asking().get() {
                        agent.asking().set(false);
                        agent.error().set(Some(error.clone()));
                    }
                }
                if tag == "character_growth_preview_trait_delta" {
                    self.store.growth().previewing().set(false);
                    self.store.growth().error().set(Some(error.clone()));
                }
                if tag == "character_growth_create_record" {
                    self.store.growth().saving().set(false);
                    self.store.growth().error().set(Some(error.clone()));
                }
                self.set_status(error, StatusTone::Error);
            }
        }
    }

    fn write_outcome(&mut self, tag: &str, outcome: CommandOutcome) -> String {
        match outcome {
            CommandOutcome::Value(value) => match tag {
                // 智能体回答：AgentAskResponse 只有 Serialize，手动取值。
                "ai_agent_ask" => {
                    let references = value
                        .get("references")
                        .and_then(|v| v.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .map(|item| crate::state::AgentReference {
                                    title: item
                                        .get("title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    entity_type: item
                                        .get("entityType")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    entity_id: item
                                        .get("entityId")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    snippet: item
                                        .get("snippet")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    source: item
                                        .get("source")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    score: item
                                        .get("score")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or_default()
                                        as f32,
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let provider_label = match value
                        .get("providerKind")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                    {
                        "codexCli" => "Codex CLI",
                        "claudeCli" => "Claude CLI",
                        _ => "OpenAI 兼容",
                    };
                    let answer = crate::state::AgentAnswer {
                        answer: value
                            .get("answer")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        provider_label: provider_label.to_string(),
                        model: value
                            .get("model")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        references,
                    };
                    let agent = self.store.agent();
                    agent.asking().set(false);
                    agent.answer().set(Some(answer));
                    "智能体回答已生成".into()
                }
                // 提供方连通性测试结果。
                t if t.starts_with("ai_test_") => {
                    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                    let message = value
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                        .or_else(|| {
                            value
                                .get("error")
                                .and_then(|v| v.get("message"))
                                .and_then(|v| v.as_str())
                                .map(str::to_string)
                        })
                        .unwrap_or_else(|| "测试完成".into());
                    let outcome = crate::state::TestOutcome {
                        ok,
                        message: message.clone(),
                        running: false,
                    };
                    let kind = test_kind_label(t);
                    self.store.ai_settings().test_results().update(|results| {
                        results.retain(|(existing, _)| existing != kind);
                        results.push((kind.to_string(), outcome));
                    });
                    message
                }
                // diagnostics_summary 走 Value 通道。
                _ => {
                    if let (Some(app_version), Some(platform)) = (
                        value.get("appVersion").and_then(|v| v.as_str()),
                        value.get("platform").and_then(|v| v.as_str()),
                    ) {
                        self.store
                            .diagnostics()
                            .app_version()
                            .set(Some(app_version.to_string()));
                        self.store
                            .diagnostics()
                            .platform()
                            .set(Some(platform.to_string()));
                        self.store.diagnostics().database().set(
                            value
                                .get("database")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                        );
                        return "诊断信息已加载".into();
                    }
                    "命令完成".into()
                }
            },
            CommandOutcome::Projects(projects) => {
                let has_active = self.store.projects().active_id().get().is_some();
                if !has_active {
                    if let Some(first) = projects.first() {
                        self.store
                            .projects()
                            .active_id()
                            .set(Some(first.id.clone()));
                    }
                }
                let count = projects.len();
                self.store.projects().list().set(projects);
                format!("已加载 {count} 个项目")
            }
            CommandOutcome::LibrarySnapshot(snapshot) => {
                let library = self.store.library();
                library.entries().set(snapshot.entries);
                library.characters().set(snapshot.characters);
                library.events().set(snapshot.events);
                library.axioms().set(snapshot.axioms);
                library.relations().set(snapshot.relations);
                library.neighborhood().set(None);
                "资料库快照已加载".into()
            }
            CommandOutcome::Jobs(jobs) => {
                let count = jobs.len();
                self.store.jobs().list().set(jobs);
                format!("已加载 {count} 条任务")
            }
            CommandOutcome::CurrentJob(job) => {
                self.store.jobs().current().set(job);
                "当前任务已刷新".into()
            }
            CommandOutcome::JobLogs(logs) => {
                let rendered = logs
                    .iter()
                    .map(|log| format!("[{}] {}", log.created_at, log.message))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.store.jobs().open_logs().set(Some(rendered));
                "任务日志已加载".into()
            }
            CommandOutcome::Prompts(templates) => {
                let count = templates.len();
                self.store.prompts().templates().set(templates);
                format!("已加载 {count} 个模板")
            }
            CommandOutcome::PromptPreview(preview) => {
                self.store.prompts().preview().set(Some(preview));
                "模板预览已更新".into()
            }
            CommandOutcome::RelationPresets(presets) => {
                let count = presets.len();
                self.store.library().relation_presets().set(presets);
                format!("已加载 {count} 个关系预设")
            }
            CommandOutcome::Neighborhood(neighborhood) => {
                self.store.library().neighborhood().set(Some(*neighborhood));
                "关系邻域已加载".into()
            }
            CommandOutcome::ProviderSettings(views) => {
                let count = views.len();
                // UI 编辑草稿从视图镜像生成（密钥不回显，改动时才写入）。
                let drafts = views
                    .iter()
                    .map(|view| crate::state::AiSettingsDraft {
                        kind: provider_kind_key(&view.kind),
                        label: provider_kind_label(&view.kind).to_string(),
                        base_url: view.base_url.clone().unwrap_or_default(),
                        api_key: String::new(),
                        clear_api_key: false,
                        chat_model: view.chat_model.clone().unwrap_or_default(),
                        embedding_model: view.embedding_model.clone().unwrap_or_default(),
                        command_template: view.command_template.clone().unwrap_or_default(),
                        enabled: view.enabled,
                    })
                    .collect::<Vec<_>>();
                self.store.ai_settings().drafts().set(drafts);
                self.store.ai_settings().views().set(views);
                self.store.ai_settings().loaded().set(true);
                format!("已加载 {count} 项提供方配置")
            }
            CommandOutcome::RagStatus(status) => {
                let ready =
                    status.missing_embedding_count == 0 && status.stale_embedding_count == 0;
                let phase = if ready {
                    crate::state::IndexPhase::Ready
                } else if status.embedding_count > 0 {
                    crate::state::IndexPhase::Degraded
                } else {
                    crate::state::IndexPhase::Idle
                };
                self.store.indexing().phase().set(phase);
                self.store.indexing().status().set(Some(status));
                "索引状态已刷新".into()
            }
            CommandOutcome::SearchResults(results) => {
                self.store.search().keyword_results().set(results);
                "关键词检索完成".into()
            }
            CommandOutcome::SemanticSearch(response) => {
                let label = match response.status {
                    ameya_core::services::search::SemanticSearchStatus::Ready => "语义检索完成",
                    ameya_core::services::search::SemanticSearchStatus::Degraded => {
                        "已回退到关键词检索"
                    }
                };
                self.store.search().semantic().set(Some(response));
                label.into()
            }
            CommandOutcome::GrowthWorkspace(workspace) => {
                let characters = workspace.states.len();
                self.store.growth().workspace().set(Some(*workspace));
                format!("成长工作台已加载（{characters} 个角色）")
            }
            CommandOutcome::TraitPreview(preview) => {
                self.store.growth().preview().set(Some(preview));
                self.store.growth().previewing().set(false);
                "特质变化预览已生成".into()
            }
            CommandOutcome::SimulationReport(report) => {
                self.store.simulation().running().set(false);
                self.store.simulation().report().set(Some(report));
                "模拟完成".into()
            }
            CommandOutcome::AuditConflicts(conflicts, repairs) => {
                self.store.audit().conflicts().set(conflicts);
                self.store.audit().repairs().set(repairs);
                "规则审计完成".into()
            }
            CommandOutcome::AuditRepairs(repairs) => {
                let count = repairs.len();
                self.store.audit().repairs().set(repairs);
                format!("已生成 {count} 条修复建议")
            }
            CommandOutcome::ArchiveText(text) => {
                self.store.backup().export_text().set(text);
                self.store.backup().busy().set(false);
                "归档已导出".into()
            }
            CommandOutcome::Diagnostics {
                app_version,
                platform,
                database,
            } => {
                self.store
                    .diagnostics()
                    .app_version()
                    .set(Some(app_version));
                self.store.diagnostics().platform().set(Some(platform));
                self.store.diagnostics().database().set(Some(database));
                "诊断信息已刷新".into()
            }
            CommandOutcome::Notice(notice) => notice,
        }
    }
}

/// 提供方种类 → 契约键（camelCase，与 core serde 一致）。
fn provider_kind_key(kind: &ameya_core::ai::settings::AiProviderKind) -> String {
    match kind {
        ameya_core::ai::settings::AiProviderKind::OpenAiCompatible => "openAiCompatible",
        ameya_core::ai::settings::AiProviderKind::CodexCli => "codexCli",
        ameya_core::ai::settings::AiProviderKind::ClaudeCli => "claudeCli",
    }
    .to_string()
}

fn provider_kind_label(kind: &ameya_core::ai::settings::AiProviderKind) -> &'static str {
    match kind {
        ameya_core::ai::settings::AiProviderKind::OpenAiCompatible => "OpenAI 兼容",
        ameya_core::ai::settings::AiProviderKind::CodexCli => "Codex CLI",
        ameya_core::ai::settings::AiProviderKind::ClaudeCli => "Claude CLI",
    }
}

/// 测试命令 → 对应提供方键（与设置草稿 kind 对齐）。
fn test_kind_label(command: &str) -> &'static str {
    match command {
        "ai_test_openai_provider" => "openAiCompatible",
        "ai_test_codex_cli_provider" => "codexCli",
        _ => "claudeCli",
    }
}

/// 新建对象草稿：按当前集合给出 Tauri 版的默认值（新词条 N+1 等）。
fn new_record_defaults(state: &AppState) -> crate::state::LibraryEditorDraft {
    use crate::state::Collection;
    let mut draft = crate::state::LibraryEditorDraft::default();
    let library = &state.library;
    let next = |count: usize| format!("{}", count + 1);
    match library.collection.unwrap_or(Collection::Entry) {
        Collection::Entry => {
            draft.title = format!("未命名词条 {}", next(library.entries.len()));
            draft.entry_type = "概念".into();
            draft.status = "draft".into();
        }
        Collection::Character => {
            draft.name = format!("未命名角色 {}", next(library.characters.len()));
        }
        Collection::Event => {
            draft.title = format!("未命名事件 {}", next(library.events.len()));
        }
        Collection::Axiom => {
            draft.subject = "新主体".into();
            draft.predicate = "定义".into();
            draft.object = "新对象".into();
            draft.certainty = 1.0;
        }
        Collection::Relation => {
            draft.relation_type = "关联".into();
            draft.confidence = 1.0;
            draft.directed = true;
            if let (Some(source), Some(target)) = (
                library.entries.first().map(|entry| (&entry.id, "entry")),
                library.entries.get(1).map(|entry| (&entry.id, "entry")),
            ) {
                draft.relation_source = format!("entry:{}", source.0);
                draft.relation_target = format!("entry:{}", target.0);
            }
        }
    }
    draft
}

/// 编辑器文本字段写回。
fn apply_editor_field(
    editor: &mut crate::state::LibraryEditorDraft,
    field: EditorField,
    value: String,
) {
    match field {
        EditorField::Title => editor.title = value,
        EditorField::EntryType => editor.entry_type = value,
        EditorField::Status => editor.status = value,
        EditorField::Tags => editor.tags = value,
        EditorField::Summary => editor.summary = value,
        EditorField::Body => editor.body = value,
        EditorField::Name => editor.name = value,
        EditorField::Faction => editor.faction = value,
        EditorField::Aliases => editor.aliases = value,
        EditorField::Appearance => editor.appearance = value,
        EditorField::Goals => editor.goals = value,
        EditorField::Motivations => editor.motivations = value,
        EditorField::Fears => editor.fears = value,
        EditorField::TimeLabel => editor.time_label = value,
        EditorField::Location => editor.location = value,
        EditorField::StartLabel => editor.start_label = value,
        EditorField::EndLabel => editor.end_label = value,
        EditorField::Description => editor.description = value,
        EditorField::Outcome => editor.outcome = value,
        EditorField::Subject => editor.subject = value,
        EditorField::Predicate => editor.predicate = value,
        EditorField::Object => editor.object = value,
        EditorField::ScopeTime => editor.scope_time = value,
        EditorField::ScopeLocation => editor.scope_location = value,
        EditorField::SourceEntityType => editor.source_entity_type = value,
        EditorField::SourceEntityId => editor.source_entity_id = value,
        EditorField::NaturalLanguage => editor.natural_language = value,
        EditorField::RelationSource => editor.relation_source = value,
        EditorField::RelationTarget => editor.relation_target = value,
        EditorField::RelationType => editor.relation_type = value,
    }
}

impl AmeyaApp {
    /// 显式载荷命令（任务按钮 / 建议确认等不经过 store 选择态的调用）。
    fn run_payload_command(
        &mut self,
        command: &'static str,
        label: &'static str,
        payload: serde_json::Value,
        context: &RuntimeProgramContext<AppMessage>,
    ) -> RuntimeProgramUpdate {
        let Some(host) = self.host.clone() else {
            self.set_status("本地数据目录不可用，命令未执行", StatusTone::Error);
            return RuntimeProgramUpdate::redraw_all();
        };
        self.push_busy(command, label);
        let result = host
            .invoke(command, payload)
            .map(|value| commands::parse_outcome(command, value));
        let message = AppMessage::CommandDone(Box::new(crate::state::CommandDone {
            tag: command,
            outcome: result,
        }));
        let _ = context.dispatch(message);
        RuntimeProgramUpdate::redraw_all()
    }

    /// 任务命令（按钮直接携带任务 ID）。
    fn run_job_command(
        &mut self,
        command: &'static str,
        label: &'static str,
        job_id: String,
        context: &RuntimeProgramContext<AppMessage>,
    ) -> RuntimeProgramUpdate {
        self.run_payload_command(command, label, commands::job_payload(&job_id), context)
    }

    /// 命令完成后的联动刷新（跟随数据变化重拉相关域）。
    fn after_command(&mut self, tag: &'static str, context: &RuntimeProgramContext<AppMessage>) {
        match tag {
            "project_create" => {
                self.store.projects().create_name().set(String::new());
                self.store
                    .projects()
                    .create_description()
                    .set(String::new());
                let _ = self.run_local_command("project_list", "刷新项目", context);
            }
            "archive_import_project" => {
                let _ = self.run_local_command("project_list", "刷新项目", context);
                let _ = self.run_local_command("library_project_snapshot", "刷新资料库", context);
            }
            "search_entities" => {
                let asking = self.store.agent().asking().get();
                if asking {
                    let _ = self.spawn_network_command("ai_agent_ask", "智能体回答中", context);
                } else {
                    let _ = self.spawn_network_command("search_semantic", "语义检索", context);
                }
            }
            "logic_audit_facts" => {
                let has_conflicts = !self.store.audit().conflicts().get().is_empty();
                if has_conflicts {
                    let _ =
                        self.run_local_command("logic_repair_suggestions", "生成修复建议", context);
                }
            }
            "prompt_save_template" => {
                self.store.prompts().notice().set(Some("模板已保存".into()));
                let _ = self.run_local_command("prompt_list_templates", "刷新模板", context);
            }
            "prompt_reset_builtin_templates" => {
                self.store
                    .prompts()
                    .notice()
                    .set(Some("内置模板已重置".into()));
                let _ = self.run_local_command("prompt_list_templates", "刷新模板", context);
            }
            "ai_save_provider_settings" => {
                let _ =
                    self.run_local_command("ai_load_provider_settings", "刷新提供方配置", context);
            }
            "job_cancel" | "job_retry" => {
                let _ = self.run_local_command("job_list", "刷新任务", context);
            }
            "job_create" => {
                self.store
                    .agent()
                    .save_message()
                    .set(Some("已保存为任务记录".into()));
                let _ = self.run_local_command("job_list", "刷新任务", context);
            }
            "agent_save_report" => {
                self.store
                    .agent()
                    .save_message()
                    .set(Some("已保存为报告词条".into()));
                let _ = self.run_local_command("library_project_snapshot", "刷新资料库", context);
            }
            "library_create_entry"
            | "library_create_character"
            | "library_create_event"
            | "library_create_axiom"
            | "library_create_relation"
            | "library_update_entry"
            | "library_update_character"
            | "library_update_event"
            | "library_update_axiom"
            | "library_update_relation" => {
                let _ = self.run_local_command("library_project_snapshot", "刷新资料库", context);
                if tag == "library_create_relation" {
                    let selected = self.store.library().selected().get();
                    if selected.is_some() {
                        let _ = self.run_local_command(
                            "library_relation_neighborhood",
                            "刷新关系邻域",
                            context,
                        );
                    }
                }
            }
            "library_delete_entry"
            | "library_delete_character"
            | "library_delete_event"
            | "library_delete_axiom"
            | "library_delete_relation" => {
                self.store.library().selected().set(None);
                self.store.library().neighborhood().set(None);
                let _ = self.run_local_command("library_project_snapshot", "刷新资料库", context);
            }
            "character_growth_create_record" => {
                self.store.growth().saving().set(false);
                self.store
                    .growth()
                    .notice()
                    .set(Some("成长记录已保存".into()));
                let _ =
                    self.run_local_command("character_growth_workspace", "刷新成长工作台", context);
            }
            "rag_index_embeddings" => {
                self.store.indexing().building().set(false);
                let _ = self.run_local_command("rag_index_status", "刷新索引状态", context);
            }
            _ => {}
        }
    }
}

impl ApplicationState for AmeyaApp {
    type Message = AppMessage;
    type Error = FrameworkError;
    /// 视图全部由 Rust 构造器创建，无需 tag 绑定内建组件。
    const BUILTINS: BuiltinComponents = BuiltinComponents::Typed;

    fn initialize(context: &RuntimeProgramContext<Self::Message>) -> Result<Self, Self::Error> {
        let store = store(AppState::default());
        // 恢复持久化的主题与侧栏折叠状态（main.rs 挂载的 SharedStore）。
        if let Ok(Some(theme)) = context.store().get("ameya.theme") {
            if theme == "light" {
                store.theme().set(ThemeChoice::Light);
            }
        }
        if let Ok(Some(collapsed)) = context.store().get("ameya.sidebarCollapsed") {
            if collapsed == "true" {
                store.sidebar_collapsed().set(true);
            }
        }
        // 持久化的侧栏宽度（Tauri 契约 180–480；非法值回默认 240）。
        let initial_sidebar_width = context
            .store()
            .get("ameya.sidebarWidth")
            .ok()
            .flatten()
            .and_then(|raw| raw.parse::<f32>().ok())
            .filter(|width| (shell::SIDEBAR_MIN_WIDTH..=shell::SIDEBAR_MAX_WIDTH).contains(width));
        // 数据目录：优先 NanaUI 解析的应用路径，回落到临时目录（诊断页可见）。
        let host = (|| {
            let (data_dir, config_dir) = match NanaApplication::paths() {
                Some(paths) => (paths.data().to_path_buf(), paths.config().to_path_buf()),
                None => {
                    let fallback = std::env::temp_dir().join("ameya");
                    (fallback.clone(), fallback)
                }
            };
            NativeCommandHost::initialize_with_config(data_dir, Some(config_dir)).ok()
        })();
        if host.is_none() {
            store.status().set(StatusLine {
                text: "本地数据目录初始化失败，功能受限".into(),
                tone: StatusTone::Warning,
            });
        }
        // 离线快照模式：注入确定性的示例数据，让真实视图在无用户交互的
        // headless 环境里也能渲染出完整布局（列表、编辑器、状态卡）。
        if std::env::var("AMEYA_SNAPSHOT_SEED").as_deref() == Ok("1") {
            crate::seed::seed_snapshot_store(&store);
        }
        Ok(Self {
            store,
            host: host.map(Arc::new),
            palette_host: entity_ref::<OverlayHost>(),
            palette: entity_ref::<CommandPalette>(),
            shell: entity_ref::<DesktopShell>(),
            initial_sidebar_width,
            last_resize_click: None,
        })
    }

    fn build(
        &mut self,
        window: &mut ApplicationWindow,
        context: &RuntimeProgramContext<Self::Message>,
    ) -> Result<(), Self::Error> {
        let store = self.store.clone();
        let context_for_view = context.clone();
        let palette_host = self.palette_host.clone();
        let palette = self.palette.clone();
        let shell_ref = self.shell.clone();
        let initial_sidebar_width = self.initial_sidebar_width;
        let document = window.document.document();
        window
            .document
            .context_mut()
            .mount_view_root(document, move || {
                shell::shell(
                    store.clone(),
                    context_for_view.clone(),
                    palette_host.clone(),
                    palette.clone(),
                    shell_ref.clone(),
                    initial_sidebar_width,
                )
            })?;
        // 恢复持久化的侧栏折叠（真值在 WorkspaceModel，需要实体已挂载）。
        if self.store.sidebar_collapsed().get() {
            if let Some(entity) = self.workspace_entity_in(window) {
                let _ = window
                    .document
                    .context_mut()
                    .update_component(entity, |workspace, _| {
                        workspace.model.update(
                            WorkspaceMutation::SetRegionCollapsed(RegionId::Resources, true),
                            Self::workspace_clock(),
                        );
                    });
            }
        }
        // 首屏数据：项目 / 诊断 / 模板 / 提供方配置 / 关系预设。
        for message in [
            AppMessage::LoadProjects,
            AppMessage::LoadDiagnostics,
            AppMessage::LoadPrompts,
            AppMessage::LoadProviderSettings,
            AppMessage::LoadRelationPresets,
        ] {
            let _ = context.dispatch(message);
        }
        Ok(())
    }

    fn update(
        &mut self,
        message: Self::Message,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
        context: &RuntimeProgramContext<Self::Message>,
    ) -> RuntimeProgramUpdate {
        match message {
            AppMessage::Navigate(route) => {
                self.store.route().set(route);
                self.set_status(STATUS_OK, StatusTone::Neutral);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::ToggleSidebar => {
                // 折叠真值在 WorkspaceModel（Resources 区）；store 字段只做启动恢复意图。
                let collapsed_now = self.toggle_sidebar_region(context.window().id(), windows);
                if let Some(collapsed) = collapsed_now {
                    self.store.sidebar_collapsed().set(collapsed);
                    let _ = context
                        .store()
                        .set("ameya.sidebarCollapsed", collapsed.to_string());
                }
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::TogglePalette => {
                match self.toggle_palette(context.window().id(), windows) {
                    Ok(update) => update,
                    Err(error) => {
                        self.set_status(format!("打开命令面板失败：{error}"), StatusTone::Error);
                        RuntimeProgramUpdate::redraw_all()
                    }
                }
            }
            AppMessage::PaletteCommand(id) => {
                if let Some(message) = shell::palette_message(id) {
                    let _ = context.dispatch(message);
                }
                if self.store.palette_open().get() {
                    let _ = self.toggle_palette(context.window().id(), windows);
                }
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::SetTheme(theme) => {
                let key = match theme {
                    ThemeChoice::Dark => "dark",
                    ThemeChoice::Light => "light",
                };
                let _ = context.store().set("ameya.theme", key.to_string());
                self.store.theme().set(theme);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::SetSettingsTab(tab) => {
                self.store.settings_tab().set(tab);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::Quit => RuntimeProgramUpdate::exit(),
            AppMessage::LoadProjects => self.run_local_command("project_list", "加载项目", context),
            AppMessage::LoadLibrary => {
                self.run_local_command("library_project_snapshot", "加载资料库", context)
            }
            AppMessage::LoadJobs => self.run_local_command("job_list", "加载任务", context),
            AppMessage::LoadPrompts => {
                self.run_local_command("prompt_list_templates", "加载模板", context)
            }
            AppMessage::LoadRelationPresets => {
                self.run_local_command("library_relation_type_presets", "加载关系预设", context)
            }
            AppMessage::LoadDiagnostics => {
                self.run_local_command("diagnostics_summary", "加载诊断", context)
            }
            AppMessage::LoadProviderSettings => {
                self.run_local_command("ai_load_provider_settings", "加载提供方配置", context)
            }
            AppMessage::LoadIndexingStatus => {
                self.run_local_command("rag_index_status", "加载索引状态", context)
            }
            AppMessage::LoadAudit => {
                self.run_local_command("logic_audit_facts", "运行样例审计", context)
            }
            AppMessage::LoadNeighborhood => {
                self.run_local_command("library_relation_neighborhood", "加载关系邻域", context)
            }
            AppMessage::SearchSubmit => {
                self.run_local_command("search_entities", "关键词检索", context)
            }
            AppMessage::AgentAsk => {
                // 先用问题跑关键词检索（引用资料），再发起网络问答。
                self.store.agent().asking().set(true);
                let query = self.store.agent().question().get();
                self.store.search().query().set(query);
                self.run_local_command("search_entities", "检索引用资料", context)
            }
            AppMessage::AgentSaveReport => {
                self.run_local_command("agent_save_report", "保存报告词条", context)
            }
            AppMessage::AgentSaveJob => {
                self.run_local_command("job_create", "保存任务记录", context)
            }
            AppMessage::SimulationRun => {
                self.run_local_command("simulation_run", "运行模拟", context)
            }
            AppMessage::BackupExport => {
                self.store.backup().busy().set(true);
                self.run_local_command("archive_export_project", "导出归档", context)
            }
            AppMessage::BackupImport => {
                self.run_local_command("archive_import_project", "导入归档", context)
            }
            AppMessage::IndexingBuild => {
                self.store.indexing().building().set(true);
                self.spawn_network_command("rag_index_embeddings", "构建嵌入索引", context)
            }
            AppMessage::JobCancel(job_id) => {
                self.run_job_command("job_cancel", "请求取消任务", job_id, context)
            }
            AppMessage::JobRetry(job_id) => {
                self.run_job_command("job_retry", "重试任务", job_id, context)
            }
            AppMessage::JobOpenLogs(job_id) => {
                self.store.jobs().open_logs().set(Some(String::new()));
                self.run_job_command("job_logs", "加载任务日志", job_id, context)
            }
            AppMessage::JobCloseLogs => {
                self.store.jobs().open_logs().set(None);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::PromptSelect(template_id) => {
                self.store
                    .prompts()
                    .selected_id()
                    .set(Some(template_id.clone()));
                self.store
                    .prompts()
                    .update(|prompts| prompts.fill_draft_from_selected());
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::PromptNew => {
                self.store
                    .prompts()
                    .update(|prompts| prompts.start_new_draft());
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::PromptSave => {
                self.run_local_command("prompt_save_template", "保存模板", context)
            }
            AppMessage::PromptPreview => {
                self.run_local_command("prompt_preview_template", "生成模板预览", context)
            }
            AppMessage::PromptReset => {
                self.run_local_command("prompt_reset_builtin_templates", "重置内置模板", context)
            }
            AppMessage::PromptVariableValue(index, value) => {
                self.store.prompts().variable_values().update(|values| {
                    if let Some(slot) = values.get_mut(index) {
                        slot.1 = value;
                    }
                });
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::AiSettingsSave => {
                self.run_local_command("ai_save_provider_settings", "保存提供方设置", context)
            }
            AppMessage::AiSettingsTest(kind) => {
                let command = match kind.as_str() {
                    "codexCli" => "ai_test_codex_cli_provider",
                    "claudeCli" => "ai_test_claude_cli_provider",
                    _ => "ai_test_openai_provider",
                };
                self.store.ai_settings().test_results().update(|results| {
                    results.retain(|(existing, _)| existing != &kind);
                    results.push((
                        kind.clone(),
                        crate::state::TestOutcome {
                            ok: false,
                            message: "测试中…".into(),
                            running: true,
                        },
                    ));
                });
                self.spawn_network_command(command, "测试提供方", context)
            }
            AppMessage::AiSettingsToggle(kind) => {
                self.store.ai_settings().drafts().update(|drafts| {
                    if let Some(draft) = drafts.iter_mut().find(|draft| draft.kind == kind) {
                        draft.enabled = !draft.enabled;
                    }
                });
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::AiSettingsField(field) => {
                self.store.ai_settings().drafts().update(|drafts| {
                    if let Some(draft) = drafts.iter_mut().find(|draft| draft.kind == field.kind) {
                        match field.field {
                            crate::state::AiSettingsFieldKind::BaseUrl => {
                                draft.base_url = field.value
                            }
                            crate::state::AiSettingsFieldKind::ApiKey => {
                                draft.api_key = field.value
                            }
                            crate::state::AiSettingsFieldKind::ChatModel => {
                                draft.chat_model = field.value
                            }
                            crate::state::AiSettingsFieldKind::EmbeddingModel => {
                                draft.embedding_model = field.value
                            }
                            crate::state::AiSettingsFieldKind::CommandTemplate => {
                                draft.command_template = field.value
                            }
                        }
                    }
                });
                RuntimeProgramUpdate::redraw_all()
            }
            // —— 对象工作台（Projects 页）——
            AppMessage::SelectProject(project_id) => {
                self.store.projects().active_id().set(Some(project_id));
                let _ = context.dispatch(AppMessage::LoadLibrary);
                let _ = context.dispatch(AppMessage::LoadGrowthWorkspace);
                let _ = context.dispatch(AppMessage::LoadIndexingStatus);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::ProjectNameDraftChanged(name) => {
                self.store.projects().create_name().set(name);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::ProjectDescriptionDraftChanged(description) => {
                self.store.projects().create_description().set(description);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::CreateProject | AppMessage::HomeCreateProject => {
                self.run_local_command("project_create", "新建项目", context)
            }
            AppMessage::SelectCollection(collection) => {
                self.store.library().collection().set(Some(collection));
                self.store.library().selected().set(None);
                self.store.library().neighborhood().set(None);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::SelectObject(selected) => {
                self.store.library().selected().set(selected.clone());
                match selected {
                    Some(_) => self.run_local_command(
                        "library_relation_neighborhood",
                        "加载关系邻域",
                        context,
                    ),
                    None => {
                        self.store.library().neighborhood().set(None);
                        RuntimeProgramUpdate::redraw_all()
                    }
                }
            }
            AppMessage::NewRecord => {
                let defaults = self.store.with(|state| new_record_defaults(state));
                self.store
                    .library()
                    .editor()
                    .update(|editor| *editor = defaults);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::SaveRecord => {
                let command = self
                    .store
                    .with(|state| match state.library.selected.as_ref() {
                        Some(selected) => match selected.kind {
                            Collection::Entry => "library_update_entry",
                            Collection::Character => "library_update_character",
                            Collection::Event => "library_update_event",
                            Collection::Axiom => "library_update_axiom",
                            Collection::Relation => "library_update_relation",
                        },
                        None => match state.library.collection {
                            Some(Collection::Character) => "library_create_character",
                            Some(Collection::Event) => "library_create_event",
                            Some(Collection::Axiom) => "library_create_axiom",
                            Some(Collection::Relation) => "library_create_relation",
                            _ => "library_create_entry",
                        },
                    });
                let label = if command.contains("update") {
                    "保存对象改动"
                } else {
                    "新建对象"
                };
                self.run_local_command(command, label, context)
            }
            AppMessage::DeleteRecord => {
                let command = self
                    .store
                    .with(|state| match state.library.selected.as_ref() {
                        Some(selected) => match selected.kind {
                            Collection::Entry => "library_delete_entry",
                            Collection::Character => "library_delete_character",
                            Collection::Event => "library_delete_event",
                            Collection::Axiom => "library_delete_axiom",
                            Collection::Relation => "library_delete_relation",
                        },
                        None => "",
                    });
                if command.is_empty() {
                    self.set_status("请先选择要删除的对象", StatusTone::Error);
                    return RuntimeProgramUpdate::redraw_all();
                }
                self.run_local_command(command, "删除对象", context)
            }
            AppMessage::ConfirmSuggestion(suggestion) => {
                let project_id = self
                    .store
                    .with(|state| state.active_project_id().map(str::to_string))
                    .unwrap_or_default();
                let draft = serde_json::json!({
                    "projectId": project_id,
                    "source": {"entityType": suggestion.source.entity_type, "entityId": suggestion.source.entity_id},
                    "target": {"entityType": suggestion.target.entity_type, "entityId": suggestion.target.entity_id},
                    "relationType": suggestion.relation_type,
                    "description": suggestion.description,
                    "confidence": suggestion.confidence,
                    "directed": suggestion.directed,
                });
                self.run_payload_command(
                    "library_create_relation",
                    "确认关系建议",
                    serde_json::json!({"draft": draft}),
                    context,
                )
            }
            AppMessage::EditorChanged(field, value) => {
                self.store.library().editor().update(|editor| {
                    apply_editor_field(editor, field, value);
                });
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::EditorNumberChanged(field, value) => {
                self.store.library().editor().update(|editor| match field {
                    EditorNumberField::SortKey => editor.sort_key = value,
                    EditorNumberField::Importance => editor.importance = value,
                    EditorNumberField::Certainty => editor.certainty = value,
                    EditorNumberField::Confidence => editor.confidence = value,
                });
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::EditorDirectedToggled(directed) => {
                self.store
                    .library()
                    .editor()
                    .update(|editor| editor.directed = directed);
                RuntimeProgramUpdate::redraw_all()
            }
            // —— 图谱页 ——
            AppMessage::GraphRelationFilterChanged(filter) => {
                self.store.graph().relation_filter().set(filter);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GraphEntityFilterChanged(filter) => {
                self.store.graph().entity_filter().set(filter);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GraphFocusChanged(focus) => {
                self.store.graph().focus().set(focus);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GraphViewportChanged(viewport) => {
                self.store.graph().viewport().set(viewport);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GraphViewportReset => {
                self.store
                    .graph()
                    .viewport()
                    .set(crate::state::default_graph_viewport());
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::OpenObjectInWorkbench(target) => {
                self.store.route().set(Route::Projects);
                let _ = context.dispatch(AppMessage::SelectObject(Some(target)));
                RuntimeProgramUpdate::redraw_all()
            }
            // —— 角色成长页 ——
            AppMessage::LoadGrowthWorkspace => {
                self.run_local_command("character_growth_workspace", "加载成长工作台", context)
            }
            AppMessage::GrowthCharacterSelected(character_id) => {
                self.store.growth().selected_character().set(character_id);
                self.store.growth().preview().set(None);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GrowthSourceEventSelected(event_id) => {
                self.store.growth().source_event().set(event_id);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GrowthTraitNameChanged(name) => {
                self.store.growth().trait_name().set(name);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GrowthDeltaChanged(delta) => {
                self.store.growth().delta_text().set(delta);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GrowthReasonChanged(reason) => {
                self.store.growth().reason().set(reason);
                RuntimeProgramUpdate::redraw_all()
            }
            AppMessage::GrowthPreview => {
                self.store.growth().previewing().set(true);
                self.run_local_command(
                    "character_growth_preview_trait_delta",
                    "预览特质变化",
                    context,
                )
            }
            AppMessage::GrowthSave => {
                self.store.growth().saving().set(true);
                self.run_local_command("character_growth_create_record", "保存成长记录", context)
            }
            AppMessage::CommandDone(done) => {
                let crate::state::CommandDone { tag, outcome } = *done;
                self.apply_outcome(tag, outcome);
                self.after_command(tag, context);
                RuntimeProgramUpdate::redraw_all()
            }
        }
    }

    fn input_event(
        &mut self,
        id: WindowId,
        input: nana_ui::RoutedInput<'_>,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
        _context: &RuntimeProgramContext<Self::Message>,
    ) -> Result<RuntimeProgramUpdate, FrameworkError> {
        if let InputPayload::Key(key) = &input.event.payload {
            let ctrl_k = key.state == KeyState::Pressed
                && !key.repeat
                && key.modifiers.control
                && !key.modifiers.alt
                && !key.modifiers.meta
                && key.logical.0.eq_ignore_ascii_case("k");
            if ctrl_k {
                return self.toggle_palette(id, windows);
            }
        }
        if let InputPayload::Pointer(pointer) = &input.event.payload {
            match pointer.phase {
                // 拖完松手：把用户拖出的宽度写入 kv。
                PointerPhase::Up | PointerPhase::Cancel => {
                    self.persist_sidebar_width(id, windows, _context);
                }
                // 手柄双击：宽度回默认 240（Tauri 双击重置契约）。
                PointerPhase::Down if pointer.button == 0 => {
                    let Some(target) = input.pointer_hit else {
                        return Ok(RuntimeProgramUpdate::default());
                    };
                    let is_handle = windows.get_mut(&id).is_some_and(|window| {
                        window
                            .document
                            .context_mut()
                            .world()
                            .node(target)
                            .is_some_and(|node| {
                                matches!(
                                    node.kind,
                                    NodeKind::Element { ref tag } if tag == "workspace-resize-handle"
                                )
                            })
                    });
                    if !is_handle {
                        return Ok(RuntimeProgramUpdate::default());
                    }
                    let now = Instant::now();
                    let is_double = self.last_resize_click.as_ref().is_some_and(|(node, at)| {
                        *node == target && now.duration_since(*at) < SIDEBAR_DOUBLE_CLICK
                    });
                    self.last_resize_click = if is_double { None } else { Some((target, now)) };
                    if is_double {
                        self.reset_sidebar_region(id, windows);
                        self.persist_sidebar_width(id, windows, _context);
                        return Ok(RuntimeProgramUpdate::redraw(id));
                    }
                }
                PointerPhase::Down | PointerPhase::Move => {}
            }
        }
        Ok(RuntimeProgramUpdate::default())
    }

    fn close_requested(
        &mut self,
        id: WindowId,
        windows: &mut std::collections::HashMap<WindowId, ApplicationWindow>,
        context: &RuntimeProgramContext<Self::Message>,
    ) -> RuntimeProgramUpdate {
        // 关窗前保存侧栏宽度（主题/折叠已在各自消息里即时落盘）。
        self.persist_sidebar_width(id, windows, context);
        RuntimeProgramUpdate {
            window_commands: vec![WindowCommand::Close(id)],
            ..RuntimeProgramUpdate::default()
        }
    }

    fn theme(&self) -> Arc<CompiledTheme> {
        match self.store.theme().get() {
            ThemeChoice::Dark => builtin_theme_arc(ThemeAppearance::Dark),
            ThemeChoice::Light => builtin_theme_arc(ThemeAppearance::Light),
        }
    }
}
