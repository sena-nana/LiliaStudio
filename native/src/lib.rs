//! NanaUI-native application model for Ameya.
//!
//! The view/builder layer deliberately owns navigation and layout only. Every
//! mutation is represented by a command name that the native adapter dispatches
//! to the existing Rust repositories; no widget reaches SQLite.

use std::fmt;

// The application domain is shared as a framework-neutral crate. Nana owns
// only the window, view, and command transport boundary.
pub use ameya_core::{db, domain, logic, services, vector};
mod host;
pub use host::{HealthInfo, NativeCommandHost};
#[cfg(feature = "nana-runtime")]
pub mod app;
#[cfg(feature = "nana-runtime")]
pub mod commands;
pub mod page_model;
#[cfg(feature = "nana-runtime")]
pub mod rich_convert;
pub mod secrets;
#[cfg(feature = "nana-runtime")]
pub mod seed;
#[cfg(feature = "nana-runtime")]
pub mod shell;
#[cfg(feature = "nana-runtime")]
pub mod snapshot_export;
#[cfg(feature = "nana-runtime")]
pub mod state;
#[cfg(feature = "nana-runtime")]
pub mod views;
pub use page_model::{
    ActionPlacement, EmptyState, FieldControlKind, ListPresentation, PageAction, PageActionKind,
    PageField, PageLayout, PageList, PageModel, PageSection, PageStatus, PageValueKind,
};

pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 800;
pub const TITLEBAR_HEIGHT: i32 = 36;
pub const SIDEBAR_WIDTH: i32 = 240;
pub const CONTENT_PADDING: i32 = 24;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub const fn contains(self, child: Rect) -> bool {
        child.x >= self.x
            && child.y >= self.y
            && child.x + child.w <= self.x + self.w
            && child.y + child.h <= self.y + self.h
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Titlebar,
    Navigation,
    Workspace,
    Inspector,
    Status,
}

impl Group {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Titlebar => "titlebar",
            Self::Navigation => "navigation",
            Self::Workspace => "workspace",
            Self::Inspector => "inspector",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    pub id: String,
    pub group: Group,
    pub rect: Rect,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub id: String,
    pub rect: Rect,
    pub groups: Vec<(Group, Rect, [i32; 4])>,
    pub controls: Vec<Control>,
}

/// Builder used by the native Nana host. It keeps group/spacing rules explicit
/// so the same output can be fed to the offscreen snapshot validator.
#[derive(Debug, Default)]
pub struct ViewBuilder {
    id: String,
    rect: Rect,
    groups: Vec<(Group, Rect, [i32; 4])>,
    controls: Vec<Control>,
}

impl ViewBuilder {
    pub fn new(id: impl Into<String>, rect: Rect) -> Self {
        Self {
            id: id.into(),
            rect,
            ..Self::default()
        }
    }
    pub fn group(mut self, group: Group, rect: Rect, margins: [i32; 4]) -> Self {
        self.groups.push((group, rect, margins));
        self
    }
    pub fn control(mut self, id: impl Into<String>, group: Group, rect: Rect) -> Self {
        self.controls.push(Control {
            id: id.into(),
            group,
            rect,
            enabled: true,
        });
        self
    }
    pub fn build(self) -> View {
        View {
            id: self.id,
            rect: self.rect,
            groups: self.groups,
            controls: self.controls,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Route {
    #[default]
    Home,
    Projects,
    Growth,
    Timeline,
    Audit,
    Graph,
    Search,
    Indexing,
    Backup,
    Simulation,
    Agent,
    Settings,
    Help,
    Diagnostics,
    Jobs,
    PromptTemplates,
}

impl Route {
    pub const ALL: [Self; 16] = [
        Self::Home,
        Self::Projects,
        Self::Growth,
        Self::Timeline,
        Self::Audit,
        Self::Graph,
        Self::Search,
        Self::Indexing,
        Self::Backup,
        Self::Simulation,
        Self::Agent,
        Self::Settings,
        Self::Help,
        Self::Diagnostics,
        Self::Jobs,
        Self::PromptTemplates,
    ];
    pub const fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Projects => "/projects",
            Self::Growth => "/growth",
            Self::Timeline => "/timeline",
            Self::Audit => "/audit",
            Self::Graph => "/graph",
            Self::Search => "/search",
            Self::Indexing => "/indexing",
            Self::Backup => "/backup",
            Self::Simulation => "/simulation",
            Self::Agent => "/agent",
            Self::Settings => "/settings",
            Self::Help => "/help",
            Self::Diagnostics => "/diagnostics",
            Self::Jobs => "/jobs",
            Self::PromptTemplates => "/prompt-templates",
        }
    }
    pub const fn title(self) -> &'static str {
        match self {
            Self::Home => "项目",
            Self::Projects => "资料库",
            Self::Growth => "角色成长",
            Self::Timeline => "事件时间线",
            Self::Audit => "规则审计",
            Self::Graph => "关系图谱",
            Self::Search => "分析与检索",
            Self::Indexing => "索引",
            Self::Backup => "备份",
            Self::Simulation => "模拟",
            Self::Agent => "智能体",
            Self::Settings => "设置",
            Self::Help => "帮助",
            Self::Diagnostics => "诊断",
            Self::Jobs => "任务",
            Self::PromptTemplates => "提示词模板",
        }
    }

    pub const fn actions(self) -> &'static [(&'static str, &'static str, &'static str)] {
        match self {
            Self::Home => &[
                ("open-project", "打开项目", "project_list"),
                ("run-check", "运行校验", "health_check"),
            ],
            Self::Projects => &[
                ("create-project", "新建项目", "project_create"),
                ("library-snapshot", "资料库快照", "library_project_snapshot"),
            ],
            Self::Growth => &[
                (
                    "preview-growth",
                    "预览成长",
                    "character_growth_preview_trait_delta",
                ),
                ("save-growth", "保存成长", "character_growth_create_record"),
            ],
            Self::Timeline => &[
                ("create-event", "新建事件", "library_create_event"),
                ("update-event", "更新事件", "library_update_event"),
            ],
            Self::Audit => &[
                ("audit-facts", "运行审计", "logic_audit_facts"),
                ("repair-suggestions", "修复建议", "logic_repair_suggestions"),
            ],
            Self::Graph => &[
                (
                    "relation-neighborhood",
                    "关系邻域",
                    "library_relation_neighborhood",
                ),
                (
                    "relation-presets",
                    "关系预设",
                    "library_relation_type_presets",
                ),
            ],
            Self::Search => &[
                ("keyword-search", "关键词检索", "search_entities"),
                ("semantic-search", "语义检索", "search_semantic"),
            ],
            Self::Indexing => &[
                ("index-chunks", "索引文本", "rag_index_chunks"),
                ("index-embeddings", "生成向量", "rag_index_embeddings"),
            ],
            Self::Backup => &[
                ("export-project", "导出项目", "archive_export_project"),
                ("import-project", "导入项目", "archive_import_project"),
            ],
            Self::Simulation => &[("run-simulation", "运行模拟", "simulation_run")],
            Self::Agent => &[
                ("ask-agent", "询问智能体", "ai_agent_ask"),
                (
                    "provider-settings",
                    "提供方设置",
                    "ai_load_provider_settings",
                ),
            ],
            Self::Settings => &[
                ("load-settings", "读取设置", "ai_load_provider_settings"),
                ("save-settings", "保存设置", "ai_save_provider_settings"),
            ],
            Self::Help => &[("health-check", "检查连接", "health_check")],
            Self::Diagnostics => &[("diagnostics", "生成诊断", "diagnostics_summary")],
            Self::Jobs => &[
                ("list-jobs", "任务列表", "job_list"),
                ("retry-job", "重试任务", "job_retry"),
            ],
            Self::PromptTemplates => &[
                ("list-prompts", "模板列表", "prompt_list_templates"),
                ("preview-prompt", "预览模板", "prompt_preview_template"),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeApp {
    pub route: Route,
    pub sidebar_collapsed: bool,
    /// Project context selected by the workspace and reused by project-scoped commands.
    pub selected_project_id: Option<String>,
    /// Last command outcome shown by the shell status surface.
    pub status_message: Option<String>,
    /// Mutable form state used to build command payloads before host invocation.
    pub project_draft_name: String,
    pub project_draft_description: String,
}

impl Default for NativeApp {
    fn default() -> Self {
        Self {
            route: Route::Home,
            sidebar_collapsed: false,
            selected_project_id: None,
            status_message: None,
            project_draft_name: "未命名项目".to_string(),
            project_draft_description: "由 NanaUI 工作区创建".to_string(),
        }
    }
}

impl NativeApp {
    pub fn view(&self) -> View {
        self.view_for(self.route)
    }

    /// Return the current route's auditable business content descriptor.
    pub fn page_model(&self) -> PageModel {
        self.route.page_model()
    }

    pub fn view_for(&self, route: Route) -> View {
        let sidebar = if self.sidebar_collapsed {
            56
        } else {
            SIDEBAR_WIDTH
        };
        let body_y = TITLEBAR_HEIGHT;
        let body_h = WINDOW_HEIGHT - TITLEBAR_HEIGHT - 28;
        let mut builder =
            ViewBuilder::new("ameya-shell", Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT))
                .group(
                    Group::Titlebar,
                    Rect::new(0, 0, WINDOW_WIDTH, TITLEBAR_HEIGHT),
                    [0, 0, 0, 0],
                )
                .group(
                    Group::Navigation,
                    Rect::new(0, body_y, sidebar, body_h),
                    [16, 16, 16, 16],
                )
                .group(
                    Group::Workspace,
                    Rect::new(sidebar, body_y, WINDOW_WIDTH - sidebar, body_h),
                    [CONTENT_PADDING, 20, CONTENT_PADDING, 20],
                )
                .group(
                    Group::Status,
                    Rect::new(sidebar, WINDOW_HEIGHT - 28, WINDOW_WIDTH - sidebar, 28),
                    [8, 8, 8, 8],
                )
                .control("title", Group::Titlebar, Rect::new(16, 8, 180, 20))
                .control(
                    "route",
                    Group::Navigation,
                    Rect::new(
                        16,
                        body_y + 24,
                        if self.sidebar_collapsed { 24 } else { 120 },
                        32,
                    ),
                )
                .control(
                    "primary",
                    Group::Workspace,
                    Rect::new(
                        sidebar + CONTENT_PADDING,
                        body_y + 20 + CONTENT_PADDING,
                        180,
                        36,
                    ),
                )
                .control(
                    "connection",
                    Group::Status,
                    Rect::new(sidebar + 16, WINDOW_HEIGHT - 20, 120, 12),
                );
        for (index, (id, _label, _command)) in route.actions().iter().enumerate() {
            let y = TITLEBAR_HEIGHT + 20 + CONTENT_PADDING + 52 * (index as i32 + 1);
            builder = builder.control(
                *id,
                Group::Workspace,
                Rect::new(sidebar + CONTENT_PADDING, y, 260, 32),
            );
        }

        // Business descriptors are visible controls as well as snapshot records.
        // Keep them compact and two-column so all routes fit both sidebar modes.
        let model = route.page_model();
        let content_x = sidebar + CONTENT_PADDING;
        let content_y =
            TITLEBAR_HEIGHT + 20 + CONTENT_PADDING + 52 * (route.actions().len() as i32 + 1) + 8;
        let content_w = WINDOW_WIDTH - sidebar - CONTENT_PADDING * 2;
        let column_w = (content_w - 8) / 2;
        let mut row = 0i32;
        for page_section in &model.sections {
            builder = builder.control(
                format!("section-{}", page_section.id),
                Group::Workspace,
                Rect::new(content_x, content_y + row * 24, content_w, 20),
            );
            row += 1;
            builder = builder.control(
                format!("status-{}", page_section.id),
                Group::Workspace,
                Rect::new(content_x, content_y + row * 24, content_w, 20),
            );
            row += 1;
            let mut detail_index = 0usize;
            for page_field in &page_section.fields {
                let column = (detail_index % 2) as i32;
                let detail_row = row + (detail_index / 2) as i32;
                builder = builder.control(
                    format!("field-{}-{}", page_section.id, page_field.id),
                    Group::Workspace,
                    Rect::new(
                        content_x + column * (column_w + 8),
                        content_y + detail_row * 24,
                        column_w,
                        20,
                    ),
                );
                detail_index += 1;
            }
            for page_list in &page_section.lists {
                let detail_row = row + (detail_index / 2) as i32;
                let column = (detail_index % 2) as i32;
                builder = builder.control(
                    format!("list-{}-{}", page_section.id, page_list.id),
                    Group::Workspace,
                    Rect::new(
                        content_x + column * (column_w + 8),
                        content_y + detail_row * 24,
                        column_w,
                        20,
                    ),
                );
                detail_index += 1;
                let detail_row = row + (detail_index / 2) as i32;
                let column = (detail_index % 2) as i32;
                builder = builder.control(
                    format!("empty-list-{}-{}", page_section.id, page_list.id),
                    Group::Workspace,
                    Rect::new(
                        content_x + column * (column_w + 8),
                        content_y + detail_row * 24,
                        column_w,
                        20,
                    ),
                );
                detail_index += 1;
            }
            let detail_row = row + (detail_index / 2) as i32;
            let column = (detail_index % 2) as i32;
            builder = builder.control(
                format!("empty-{}", page_section.id),
                Group::Workspace,
                Rect::new(
                    content_x + column * (column_w + 8),
                    content_y + detail_row * 24,
                    column_w,
                    20,
                ),
            );
            row += (detail_index / 2) as i32 + 1;
        }
        builder.build()
    }

    pub fn command_names() -> &'static [&'static str] {
        &[
            "ai_agent_ask",
            "ai_default_providers",
            "ai_load_provider_settings",
            "ai_save_provider_settings",
            "ai_test_openai_provider",
            "ai_test_codex_cli_provider",
            "ai_test_claude_cli_provider",
            "archive_export_project",
            "archive_import_project",
            "diagnostics_summary",
            "health_check",
            "job_create",
            "job_list",
            "job_current",
            "job_logs",
            "job_cancel",
            "job_retry",
            "library_project_snapshot",
            "library_create_entry",
            "library_update_entry",
            "library_delete_entry",
            "library_create_character",
            "library_update_character",
            "library_delete_character",
            "library_create_event",
            "library_update_event",
            "library_delete_event",
            "library_create_axiom",
            "library_update_axiom",
            "library_delete_axiom",
            "library_list_backlinks",
            "library_relation_type_presets",
            "library_relation_neighborhood",
            "library_create_relation",
            "library_update_relation",
            "library_delete_relation",
            "logic_audit_facts",
            "logic_repair_suggestions",
            "project_list",
            "project_create",
            "project_update",
            "project_archive",
            "prompt_list_templates",
            "prompt_copy_template",
            "prompt_save_template",
            "prompt_reset_builtin_templates",
            "prompt_preview_template",
            "rag_index_chunks",
            "rag_index_status",
            "rag_index_embeddings",
            "rag_preview_context_pack",
            "search_entities",
            "search_semantic",
            "simulation_run",
            "vector_preview_chunks",
            "character_growth_preview_trait_delta",
            "character_growth_workspace",
            "character_growth_create_record",
        ]
    }

    pub fn snapshot(&self) -> String {
        self.page_snapshot(self.route)
    }

    /// Render one route as a deterministic, client-area snapshot. The records
    /// are intentionally framework-neutral so they can be checked without a
    /// display server and consumed by screenshot tooling.
    pub fn page_snapshot(&self, route: Route) -> String {
        let view = self.view_for(route);
        let mut output = format!(
            "# page|route={}|title={}\nwindow|x=0|y=0|w={}|h={}\n",
            route.path(),
            route.title(),
            view.rect.w,
            view.rect.h
        );
        for (group, rect, m) in &view.groups {
            output.push_str(&format!(
                "group|id={}|x={}|y={}|w={}|h={}|left={}|top={}|right={}|bottom={}\n",
                group.id(),
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                m[0],
                m[1],
                m[2],
                m[3]
            ));
        }
        for control in &view.controls {
            let action = if control.id == "primary" {
                "save"
            } else {
                route
                    .actions()
                    .iter()
                    .find(|(id, _, _)| *id == control.id)
                    .map_or("none", |(_, _, command)| *command)
            };
            output.push_str(&format!(
                "control|id={}|group={}|action={}|x={}|y={}|w={}|h={}|visible=1|enabled={}\n",
                control.id,
                control.group.id(),
                action,
                control.rect.x,
                control.rect.y,
                control.rect.w,
                control.rect.h,
                i32::from(control.enabled)
            ));
        }
        output.push_str(&content_records(route));
        output
    }

    /// Apply a user-level interaction to a headless app model. This exercises
    /// the same navigation/action IDs used by the NanaUI view.
    pub fn dispatch(&mut self, action: &str) -> Result<(), String> {
        if let Some(path) = action.strip_prefix("navigate:") {
            self.route = Route::ALL
                .into_iter()
                .find(|route| route.path() == path)
                .ok_or_else(|| format!("unknown route: {path}"))?;
            return Ok(());
        }
        if action == "toggle-sidebar" {
            self.sidebar_collapsed = !self.sidebar_collapsed;
            return Ok(());
        }
        if action == "save" || action == "open-project" || action == "run-check" {
            return Ok(());
        }
        if self
            .route
            .actions()
            .iter()
            .any(|(id, _, command)| *id == action || *command == action)
        {
            return Ok(());
        }
        Err(format!("unsupported action: {action}"))
    }

    /// Dispatch a route action through the native command boundary. Navigation
    /// and shell actions stay local to the view model; route actions execute in
    /// the same host used by the NanaUI desktop runtime.
    /// Build the minimum real payload for a route action from the live shell context.
    /// The hosted runtime uses this before invoking the command boundary; an action
    /// that needs a selected project fails visibly instead of silently sending `{}`.
    pub fn action_payload(&self, action: &str) -> Result<serde_json::Value, String> {
        let command = self
            .route
            .actions()
            .iter()
            .find(|(id, _, command)| *id == action || *command == action)
            .map(|(_, _, command)| *command)
            .unwrap_or(action);
        match command {
            "project_create" => {
                let name = self.project_draft_name.trim();
                if name.is_empty() {
                    return Err("项目名称不能为空".to_string());
                }
                Ok(serde_json::json!({
                    "draft": {
                        "name": name,
                        "description": self.project_draft_description.trim()
                    }
                }))
            }
            "library_project_snapshot"
            | "project_archive"
            | "project_update"
            | "archive_export_project"
            | "archive_import_project"
            | "rag_index_embeddings"
            | "simulation_run"
            | "character_growth_workspace" => {
                let project_id = self
                    .selected_project_id
                    .clone()
                    .ok_or_else(|| "请先选择一个项目".to_string())?;
                Ok(serde_json::json!({"projectId": project_id}))
            }
            "search_entities" => {
                let project_id = self
                    .selected_project_id
                    .clone()
                    .ok_or_else(|| "请先选择一个项目".to_string())?;
                Ok(serde_json::json!({
                    "filter": {"projectId": project_id, "query": "", "entityTypes": []}
                }))
            }
            "job_current" | "diagnostics_summary" | "ai_test_openai_provider" => {
                Ok(serde_json::json!({}))
            }
            "logic_audit_facts" => Ok(serde_json::json!({"facts": []})),
            _ => Ok(serde_json::json!({})),
        }
    }

    pub fn dispatch_to_host(
        &mut self,
        host: &NativeCommandHost,
        action: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if action.starts_with("navigate:")
            || matches!(
                action,
                "toggle-sidebar" | "save" | "open-project" | "run-check"
            )
        {
            self.dispatch(action)?;
            return Ok(serde_json::Value::Null);
        }
        let command = self
            .route
            .actions()
            .iter()
            .find(|(id, _, command)| *id == action || *command == action)
            .map(|(_, _, command)| *command)
            .ok_or_else(|| format!("unsupported action: {action}"))?;
        host.invoke(command, payload)
    }

    /// Write one snapshot per navigation route for offline renderers.
    pub fn render_offline_pages(
        &self,
        directory: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let directory = directory.as_ref();
        std::fs::create_dir_all(directory)?;
        for route in Route::ALL {
            let filename = route.path().trim_start_matches('/').replace('/', "-");
            let filename = if filename.is_empty() {
                "home"
            } else {
                filename.as_str()
            };
            std::fs::write(
                directory.join(format!("{filename}.snapshot")),
                self.page_snapshot(route),
            )?;
        }
        Ok(())
    }
}

/// Framework-neutral business records for one route: the compact route
/// contract (section/status/list/field/empty) plus the richer page-model
/// records and the action registry. Geometry producers (headless
/// ViewBuilder or the hosted headless runtime) append these after their
/// own window/group/control records so both stay contract-identical.
pub fn content_records(route: Route) -> String {
    let route_slug = if route.path() == "/" {
        "home"
    } else {
        route.path().trim_start_matches('/')
    };
    let mut output = String::new();
    let legacy_section = format!("{}-content", route_slug);
    let legacy_list = format!("{}-list", route_slug);
    let legacy_status = format!("{}-ready", route_slug);
    let legacy_empty = format!("{}-empty", route_slug);
    output.push_str(&format!(
        "section|id={}|group=workspace|label={} business|kind=business\n",
        legacy_section,
        route.title()
    ));
    output.push_str(&format!(
        "status|id={}|group=status|value=ready|active=1|label=可用\n",
        legacy_status
    ));
    let legacy_items = route
        .actions()
        .iter()
        .map(|(id, _, _)| *id)
        .collect::<Vec<_>>()
        .join(",");
    output.push_str(&format!(
        "list|id={}|group=workspace|items={}|empty={}|label={} actions\n",
        legacy_list,
        legacy_items,
        legacy_empty,
        route.title()
    ));
    for (id, label, command) in route.actions() {
        output.push_str(&format!(
            "field|id={}|group=workspace|label={}|value={}|state={}\n",
            id, label, command, legacy_status
        ));
    }
    output.push_str(&format!(
        "empty|id={}|group=workspace|text=暂无{}数据|visible=1\n",
        legacy_empty,
        route.title()
    ));

    let model = route.page_model();
    for page_section in &model.sections {
        output.push_str(&format!(
            "section|id={}|group=workspace|label={}|title={}|description={}\n",
            page_section.id, page_section.title, page_section.title, page_section.description
        ));
        let status_id = format!("status-{}", page_section.id);
        output.push_str(&format!(
            "status|id={}|group=status|value={}|active=1|label={}|detail={}\n",
            status_id,
            page_section.status.code,
            page_section.status.label,
            page_section.status.detail
        ));
        for page_field in &page_section.fields {
            let field_id = format!("field-{}-{}", page_section.id, page_field.id);
            let sample_value = match page_field.kind {
                PageValueKind::Identifier | PageValueKind::EntityReference => "sample-entity-001",
                PageValueKind::Text => "示例文本",
                PageValueKind::MultilineText => "示例业务内容",
                PageValueKind::Number => "3",
                PageValueKind::Decimal => "0.75",
                PageValueKind::Boolean => "否",
                PageValueKind::Timestamp => "2026-11-09T00:00:00Z",
                PageValueKind::Status => page_section.status.code,
                PageValueKind::Tags => "示例,待整理",
                PageValueKind::Json => "{}",
            };
            output.push_str(&format!(
                "field|id={}|group=workspace|label={}|value={}|state={}|kind={}|source={}|required={}\n",
                field_id,
                page_field.label,
                sample_value,
                status_id,
                page_field.kind.as_str(),
                page_field.source,
                i32::from(page_field.required)
            ));
        }
        for page_list in &page_section.lists {
            let list_id = format!("list-{}-{}", page_section.id, page_list.id);
            let empty_id = format!("empty-{}-{}", page_section.id, page_list.id);
            let columns = page_list
                .columns
                .iter()
                .map(|column| format!("{}:{}:{}", column.id, column.label, column.source))
                .collect::<Vec<_>>()
                .join(",");
            output.push_str(&format!(
                "list|id={}|group=workspace|items=sample-1,sample-2|empty={}|label={}|item={}|source={}|columns={}\n",
                list_id,
                empty_id,
                page_list.label,
                page_list.item_label,
                page_list.source,
                columns
            ));
            output.push_str(&format!(
                "empty|id={}|group=workspace|text={}|visible=1|description={}|action={}\n",
                empty_id,
                page_list.empty.title,
                page_list.empty.description,
                page_list.empty.action_id.unwrap_or("none")
            ));
        }
        let empty_id = format!("empty-{}", page_section.id);
        output.push_str(&format!(
            "empty|id={}|group=workspace|text={}|visible=1|description={}|action={}\n",
            empty_id,
            page_section.empty.title,
            page_section.empty.description,
            page_section.empty.action_id.unwrap_or("none")
        ));
    }
    for page_action in &model.actions {
        let inputs = page_action
            .inputs
            .iter()
            .map(|input| format!("{}:{}:{}", input.id, input.label, input.source))
            .collect::<Vec<_>>()
            .join(",");
        let input_suffix = if inputs.is_empty() {
            String::new()
        } else {
            format!("|inputs={inputs}")
        };
        output.push_str(&format!(
            "action|id={}|supported=1|label={}|command={}|kind={}|description={}{}\n",
            page_action.id,
            page_action.label,
            page_action.command,
            page_action.kind.as_str(),
            page_action.description,
            input_suffix
        ));
    }
    for (_id, _label, command) in route.actions() {
        output.push_str(&format!("action|id={}|supported=1\n", command));
    }
    output.push_str("action|id=save|supported=1\n");
    output
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_routes_have_native_titles_and_paths() {
        for route in Route::ALL {
            assert!(!route.title().is_empty());
            assert!(route.path().starts_with('/'));
        }
    }
    #[test]
    fn shell_groups_are_inside_window() {
        let view = NativeApp::default().view();
        for (_, rect, _) in view.groups {
            assert!(view.rect.contains(rect));
        }
    }
    #[test]
    fn controls_respect_group_content_margins() {
        for app in [
            NativeApp::default(),
            NativeApp {
                route: Route::Home,
                sidebar_collapsed: true,
                selected_project_id: None,
                status_message: None,
                ..NativeApp::default()
            },
        ] {
            let view = app.view();
            for control in view.controls {
                let (_, rect, margin) = view
                    .groups
                    .iter()
                    .find(|(group, _, _)| *group == control.group)
                    .unwrap();
                let content = Rect::new(
                    rect.x + margin[0],
                    rect.y + margin[1],
                    rect.w - margin[0] - margin[2],
                    rect.h - margin[1] - margin[3],
                );
                assert!(
                    content.contains(control.rect),
                    "{} outside {}",
                    control.id,
                    control.group.id()
                );
            }
        }
    }

    #[test]
    fn every_route_has_a_valid_offline_page_snapshot() {
        let app = NativeApp::default();
        for route in Route::ALL {
            let snapshot = app.page_snapshot(route);
            assert!(snapshot.contains(&format!("route={}", route.path())));
            assert!(snapshot.contains(&format!("title={}", route.title())));
            assert!(snapshot.matches("action|id=").count() >= 2);
            for (_, _, command) in route.actions() {
                assert!(
                    NativeApp::command_names().contains(command),
                    "unregistered command: {command}"
                );
                assert!(snapshot.contains(&format!("action={command}")));
            }
        }
    }

    #[test]
    fn every_route_business_controls_fit_both_sidebar_modes() {
        for route in Route::ALL {
            for sidebar_collapsed in [false, true] {
                let app = NativeApp {
                    route,
                    sidebar_collapsed,
                    selected_project_id: None,
                    status_message: None,
                    ..NativeApp::default()
                };
                let view = app.view();
                for control in view.controls {
                    let (_, rect, margin) = view
                        .groups
                        .iter()
                        .find(|(group, _, _)| *group == control.group)
                        .unwrap();
                    let content = Rect::new(
                        rect.x + margin[0],
                        rect.y + margin[1],
                        rect.w - margin[0] - margin[2],
                        rect.h - margin[1] - margin[3],
                    );
                    assert!(
                        content.contains(control.rect),
                        "{route:?} sidebar_collapsed={sidebar_collapsed}: {} outside {}",
                        control.id,
                        control.group.id()
                    );
                }
                let snapshot = app.page_snapshot(route);
                let model = app.page_model();
                assert!(snapshot.contains("section|id="));
                if model.field_count() > 0 {
                    assert!(
                        snapshot.contains("field|id=field-"),
                        "missing fields for {route:?}"
                    );
                }
                if model
                    .sections
                    .iter()
                    .any(|section| !section.lists.is_empty())
                {
                    assert!(snapshot.contains("list|id=list-"));
                }
                assert!(snapshot.contains("status|id="));
                assert!(snapshot.contains("empty|id="));
                assert!(snapshot.contains("action|id="));
            }
        }
    }

    #[test]
    fn project_actions_require_context_and_create_has_payload() {
        let app = NativeApp {
            route: Route::Projects,
            ..NativeApp::default()
        };
        let create = app
            .action_payload("project_create")
            .expect("create payload");
        assert_eq!(create["draft"]["name"], "未命名项目");
        assert_eq!(
            app.action_payload("library_project_snapshot"),
            Err("请先选择一个项目".to_string())
        );
        let scoped = NativeApp {
            selected_project_id: Some("project-1".to_string()),
            project_draft_name: "真实项目".to_string(),
            project_draft_description: "用户输入的描述".to_string(),
            ..app
        };
        let draft = scoped.action_payload("project_create").unwrap();
        assert_eq!(draft["draft"]["name"], "真实项目");
        assert_eq!(draft["draft"]["description"], "用户输入的描述");
        assert_eq!(
            scoped.action_payload("library_project_snapshot").unwrap()["projectId"],
            "project-1"
        );
    }

    #[test]
    fn navigation_and_page_actions_are_interactive() {
        let mut app = NativeApp::default();
        for route in Route::ALL {
            app.dispatch(&format!("navigate:{}", route.path())).unwrap();
            for (id, _, command) in route.actions() {
                app.dispatch(id).unwrap();
                app.dispatch(command).unwrap();
            }
        }
        app.dispatch("toggle-sidebar").unwrap();
        assert!(app.sidebar_collapsed);
        assert!(app.dispatch("unsupported-action").is_err());
    }
}
