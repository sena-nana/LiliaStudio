//! Ameya 全局应用状态：单一顶层 store，承载 16 条路由共享的领域数据。
//!
//! 视图层只读这里的路径（Store 按字段细粒度追踪），命令层把结果写回。
//! 网络类命令在 `app.rs` 的 update 里经 `run_task` 走后台线程，完成后
//! 以 [`AppMessage::CommandDone`] 回投；本地 SQLite 命令在 update 内联执行。

use ameya_core::{
    ai::settings::AiProviderSettingsView,
    domain::{
        axiom::Axiom, character::Character, entry::Entry, event::Event, project::Project,
        relation::Relation,
    },
    logic::conflict::LogicConflict,
    logic::repair::RepairSuggestion,
    services::{
        character_growth::{CharacterGrowthWorkspaceSnapshot, CharacterTraitState},
        jobs::{AiJob, AiJobLog},
        prompts::{PromptTemplate, PromptTemplatePreview},
        rag::RagIndexStatus,
        relation_workspace::{RelationNeighborhood, RelationSuggestion},
        search::{SearchResult, SemanticSearchResponse},
        simulation::SimulationReport,
    },
};
use nana_ui::runtime::view::Store;
use nana_ui::runtime::{GraphPoint, GraphViewport};

/// 侧栏导航分组（对齐 Tauri 版 AppShell 结构）。
pub const NAV_GROUPS: &[(&str, &[Route])] = &[
    (
        "工作区",
        &[
            Route::Home,
            Route::Projects,
            Route::Growth,
            Route::Timeline,
            Route::Audit,
            Route::Graph,
            Route::Search,
        ],
    ),
    (
        "项目工具",
        &[
            Route::Indexing,
            Route::Backup,
            Route::Simulation,
            Route::Agent,
        ],
    ),
];

/// 侧栏底部的固定链接（Tauri SidebarFooter）。
pub const NAV_FOOTER: &[Route] = &[
    Route::Settings,
    Route::Help,
    Route::Diagnostics,
    Route::Jobs,
    Route::PromptTemplates,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
}

impl ThemeChoice {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dark => "暗色",
            Self::Light => "浅色",
        }
    }
    pub const fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

/// 设置页的四个分区（Tauri SettingsSidebar）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    Appearance,
    Ai,
    CliProviders,
    About,
}

impl SettingsTab {
    pub const ALL: [Self; 4] = [Self::Appearance, Self::Ai, Self::CliProviders, Self::About];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Appearance => "外观",
            Self::Ai => "人工智能设置",
            Self::CliProviders => "命令行提供方",
            Self::About => "关于",
        }
    }
}

/// 资料库五类对象集合（Tauri LibraryCollectionRail）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Entry,
    Character,
    Event,
    Axiom,
    Relation,
}

impl Collection {
    pub const ALL: [Self; 5] = [
        Self::Entry,
        Self::Character,
        Self::Event,
        Self::Axiom,
        Self::Relation,
    ];
    /// 侧栏/浏览器的集合名（Tauri 用 资料/规则 而非 词条/公理）。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Entry => "资料",
            Self::Character => "角色",
            Self::Event => "事件",
            Self::Axiom => "规则",
            Self::Relation => "关系",
        }
    }

    /// 关系端点/邻域契约里的 entityType 键。
    pub const fn entity_key(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Character => "character",
            Self::Event => "event",
            Self::Axiom => "axiom",
            Self::Relation => "relation",
        }
    }
}

/// 资料库当前选中的对象（kind + id，供编辑器与检视器共用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRef {
    pub kind: Collection,
    pub id: String,
}

/// 对象编辑器草稿：五类对象共用一份平铺字段，按当前集合取用。
/// 摘要/正文存纯文本；落库前由 `entry_rich_text::normalize_entry_rich_text`
/// 包成 TipTap 文档，读取经 `entry_rich_text_to_plain_text` 还原。
#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct LibraryEditorDraft {
    // entry + event 共用
    pub title: String,
    pub entry_type: String,
    pub status: String,
    pub tags: String,
    pub summary: String,
    pub body: String,
    // character
    pub name: String,
    pub faction: String,
    pub aliases: String,
    pub appearance: String,
    pub goals: String,
    pub motivations: String,
    pub fears: String,
    // event
    pub time_label: String,
    pub location: String,
    pub start_label: String,
    pub end_label: String,
    pub description: String,
    pub outcome: String,
    // axiom
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub scope_time: String,
    pub scope_location: String,
    pub source_entity_type: String,
    pub source_entity_id: String,
    pub natural_language: String,
    // relation
    pub relation_source: String,
    pub relation_target: String,
    pub relation_type: String,
    pub directed: bool,
    // 数值字段（NumberInput 以 f64 驱动，落库时由命令层取整/钳制）
    pub sort_key: f64,
    pub importance: f64,
    pub certainty: f64,
    pub confidence: f64,
}

/// 编辑器文本字段标识（AppMessage::EditorChanged 的载荷）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    Title,
    EntryType,
    Status,
    Tags,
    Summary,
    Body,
    Name,
    Faction,
    Aliases,
    Appearance,
    Goals,
    Motivations,
    Fears,
    TimeLabel,
    Location,
    StartLabel,
    EndLabel,
    Description,
    Outcome,
    Subject,
    Predicate,
    Object,
    ScopeTime,
    ScopeLocation,
    SourceEntityType,
    SourceEntityId,
    NaturalLanguage,
    RelationSource,
    RelationTarget,
    RelationType,
}

/// 编辑器数值字段标识（AppMessage::EditorNumberChanged 的载荷）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorNumberField {
    SortKey,
    Importance,
    Certainty,
    Confidence,
}

/// 图谱页本地视图状态（过滤与聚焦；数据本体在 LibraryState）。
#[derive(Debug, Clone, Store)]
pub struct GraphState {
    /// 关系类型过滤；空串 = 全部关系。
    pub relation_filter: String,
    /// 实体类型过滤；空串 = 全部实体。
    pub entity_filter: String,
    /// 聚焦实体 id。
    pub focus: Option<String>,
    /// 画布视口（平移/缩放）。受控字段：重建（过滤/聚焦/数据刷新）
    /// 时原样回传，避免视口被重置；由 GraphViewportChanged 事件更新。
    pub viewport: GraphViewport,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            relation_filter: String::new(),
            entity_filter: String::new(),
            focus: None,
            viewport: default_graph_viewport(),
        }
    }
}

/// 图谱画布初始视口（zoom 0.8 是 440px 高画布下同心环的适配值）。
pub fn default_graph_viewport() -> GraphViewport {
    GraphViewport::new(GraphPoint::ZERO, GRAPH_CANVAS_ZOOM)
}

/// 图谱画布默认缩放（native/src/views/graph.rs 同心环布局配套）。
pub const GRAPH_CANVAS_ZOOM: f32 = 0.8;

use crate::Route;

/// Shell 底部状态读数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusLine {
    pub text: String,
    pub tone: StatusTone,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StatusTone {
    #[default]
    Neutral,
    Ok,
    Warning,
    Error,
}

impl StatusLine {
    pub fn neutral(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: StatusTone::Neutral,
        }
    }
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: StatusTone::Ok,
        }
    }
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: StatusTone::Error,
        }
    }
}

/// 进行中的后台操作（Spinner/禁用态依据）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusyOp {
    pub tag: &'static str,
    pub label: String,
}

/// 项目域：列表 + 当前项目 + 快速创建草稿。
#[derive(Debug, Clone, Default, Store)]
pub struct ProjectsState {
    pub list: Vec<Project>,
    pub active_id: Option<String>,
    pub loading: bool,
    pub error: Option<String>,
    pub create_name: String,
    pub create_description: String,
}

impl ProjectsState {
    pub fn active(&self) -> Option<&Project> {
        self.active_id
            .as_deref()
            .and_then(|id| self.list.iter().find(|project| project.id == id))
    }
}

/// 资料库域：五类对象快照 + 集合选择 + 对象选择 + 邻域检视数据。
#[derive(Debug, Clone, Default, Store)]
pub struct LibraryState {
    pub loading: bool,
    pub error: Option<String>,
    pub entries: Vec<Entry>,
    pub characters: Vec<Character>,
    pub events: Vec<Event>,
    pub axioms: Vec<Axiom>,
    pub relations: Vec<Relation>,
    pub collection: Option<Collection>,
    pub selected: Option<ObjectRef>,
    /// 关系类型预设（datalist 契约）。
    pub relation_presets: Vec<String>,
    /// 右栏邻域检视（深度 1）。
    pub neighborhood: Option<RelationNeighborhood>,
    /// 记录编辑器草稿。
    pub editor: LibraryEditorDraft,
}

/// 搜索域：关键词 + 语义混合结果。
#[derive(Debug, Clone, Default, Store)]
pub struct SearchState {
    pub query: String,
    pub loading: bool,
    /// 语义通道的状态展示（ready/degraded/failed + message + model）。
    pub semantic: Option<SemanticSearchResponse>,
    pub semantic_error: Option<String>,
    pub keyword_results: Vec<SearchResult>,
    pub semantic_results: Vec<SearchResult>,
}

impl SearchState {
    /// 混合结果：关键词优先，语义按 entityType:entityId 去重追加。
    pub fn mixed(&self) -> Vec<(SearchResult, bool)> {
        let mut seen = std::collections::HashSet::new();
        let mut mixed = Vec::new();
        for result in &self.keyword_results {
            if seen.insert((result.entity_type.clone(), result.entity_id.clone())) {
                mixed.push((result.clone(), false));
            }
        }
        for result in &self.semantic_results {
            if seen.insert((result.entity_type.clone(), result.entity_id.clone())) {
                mixed.push((result.clone(), true));
            }
        }
        mixed
    }
}

/// 智能体问答域。
#[derive(Debug, Clone, Default, Store)]
pub struct AgentState {
    pub question: String,
    pub asking: bool,
    pub answer: Option<AgentAnswer>,
    pub notice: Option<String>,
    pub error: Option<String>,
    /// 保存为报告/任务 的结果提示。
    pub save_message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct AgentAnswer {
    pub answer: String,
    pub provider_label: String,
    pub model: String,
    pub references: Vec<AgentReference>,
}

#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct AgentReference {
    pub title: String,
    pub entity_type: String,
    pub entity_id: String,
    pub snippet: String,
    pub source: String,
    pub score: f32,
}

/// 任务队列域。
#[derive(Debug, Clone, Default, Store)]
pub struct JobsState {
    pub list: Vec<AiJob>,
    pub current: Option<AiJob>,
    pub open_logs: Option<String>,
    pub logs: Vec<AiJobLog>,
    pub error: Option<String>,
}

/// 提示词模板域。
#[derive(Debug, Clone, Default, Store)]
pub struct PromptsState {
    pub templates: Vec<PromptTemplate>,
    pub selected_id: Option<String>,
    /// 编辑器草稿（对齐 Draft {id, name, purpose, template}）。
    pub draft_id: Option<String>,
    pub draft_name: String,
    pub draft_purpose: String,
    pub draft_template: String,
    /// 预览变量值（{{VAR}} → value）。
    pub variable_values: Vec<(String, String)>,
    pub preview: Option<PromptTemplatePreview>,
    pub error: Option<String>,
    pub notice: Option<String>,
}

/// 索引域：切片/embedding 指标与状态。
#[derive(Debug, Clone, Default, Store)]
pub struct IndexingState {
    pub status: Option<RagIndexStatus>,
    pub building: bool,
    /// 前端组合态: idle | degraded | ready | failed。
    pub phase: IndexPhase,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IndexPhase {
    #[default]
    Idle,
    Building,
    Ready,
    Degraded,
    Failed,
}

/// 角色成长域。
#[derive(Debug, Clone, Default, Store)]
pub struct GrowthState {
    pub workspace: Option<CharacterGrowthWorkspaceSnapshot>,
    pub selected_character: Option<String>,
    pub source_event: Option<String>,
    pub trait_name: String,
    pub delta_text: String,
    pub reason: String,
    pub preview: Option<CharacterTraitState>,
    pub previewing: bool,
    pub saving: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
}

/// 模拟域。
#[derive(Debug, Clone, Default, Store)]
pub struct SimulationState {
    pub report: Option<SimulationReport>,
    pub running: bool,
    pub error: Option<String>,
}

/// 规则审计域。
#[derive(Debug, Clone, Default, Store)]
pub struct AuditState {
    pub conflicts: Vec<LogicConflict>,
    pub repairs: Vec<RepairSuggestion>,
    pub error: Option<String>,
}

/// 备份导入导出域。
#[derive(Debug, Clone, Default, Store)]
pub struct BackupState {
    pub export_text: String,
    pub import_text: String,
    pub busy: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
}

/// AI 设置域（视图 + 草稿 + 测试结果）。
#[derive(Debug, Clone, Default, Store)]
pub struct AiSettingsState {
    pub views: Vec<AiProviderSettingsView>,
    /// 可编辑的草稿（从 views 派生；api_key 为空表示不修改）。
    pub drafts: Vec<AiSettingsDraft>,
    pub loaded: bool,
    pub saving: bool,
    pub message: Option<String>,
    pub error: Option<String>,
    /// 各 provider 的测试结果（kind → ok/message）。
    pub test_results: Vec<(String, TestOutcome)>,
}

/// 设置页的单个 provider 编辑草稿。kind 用 serde 名（openAiCompatible/codexCli/claudeCli）。
#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct AiSettingsDraft {
    pub kind: String,
    pub label: String,
    pub base_url: String,
    pub api_key: String,
    pub clear_api_key: bool,
    pub chat_model: String,
    pub embedding_model: String,
    pub command_template: String,
    pub enabled: bool,
}

/// 设置页一次字段编辑（文本输入回写）。
#[derive(Debug, Clone)]
pub struct AiSettingsField {
    pub kind: String,
    pub field: AiSettingsFieldKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiSettingsFieldKind {
    BaseUrl,
    ApiKey,
    ChatModel,
    EmbeddingModel,
    CommandTemplate,
}

#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct TestOutcome {
    pub ok: bool,
    pub message: String,
    pub running: bool,
}

/// 诊断域（diagnostics_summary: appVersion/platform/database）。
#[derive(Debug, Clone, Default, Store)]
pub struct DiagnosticsState {
    pub app_version: Option<String>,
    pub platform: Option<String>,
    pub database: Option<String>,
    pub error: Option<String>,
}

/// 顶层应用状态。字段即 Store 路径；嵌套结构各自 derive(Store)。
#[derive(Debug, Clone, Default, Store)]
pub struct AppState {
    // —— shell ——
    pub route: Route,
    pub sidebar_collapsed: bool,
    pub theme: ThemeChoice,
    pub palette_open: bool,
    pub status: StatusLine,
    pub busy: Vec<BusyOp>,
    // —— 设置页 ——
    pub settings_tab: SettingsTab,
    // —— 领域 ——
    pub projects: ProjectsState,
    pub library: LibraryState,
    pub graph: GraphState,
    pub search: SearchState,
    pub agent: AgentState,
    pub jobs: JobsState,
    pub prompts: PromptsState,
    pub indexing: IndexingState,
    pub growth: GrowthState,
    pub simulation: SimulationState,
    pub audit: AuditState,
    pub backup: BackupState,
    pub ai_settings: AiSettingsState,
    pub diagnostics: DiagnosticsState,
}

impl AppState {
    /// 当前项目 id（各视图共享的项目上下文）。
    pub fn active_project_id(&self) -> Option<&str> {
        self.projects.active_id.as_deref()
    }

    pub fn is_busy(&self, tag: &str) -> bool {
        self.busy.iter().any(|op| op.tag == tag)
    }

    pub fn push_busy(&mut self, tag: &'static str, label: impl Into<String>) {
        if !self.is_busy(tag) {
            self.busy.push(BusyOp {
                tag,
                label: label.into(),
            });
        }
    }

    pub fn pop_busy(&mut self, tag: &str) {
        self.busy.retain(|op| op.tag != tag);
    }
}

/// 应用消息：视图闭包 → update() 的全部入口。
#[derive(Debug, Clone)]
pub enum AppMessage {
    Navigate(Route),
    ToggleSidebar,
    TogglePalette,
    PaletteCommand(&'static str),
    SetTheme(ThemeChoice),
    SetSettingsTab(SettingsTab),
    /// 打开命令面板里选择的一条命令（ActionId 字符串）。
    Quit,

    // —— 数据装载（本地，内联执行）——
    LoadProjects,
    LoadLibrary,
    LoadJobs,
    LoadPrompts,
    LoadRelationPresets,
    LoadDiagnostics,
    LoadProviderSettings,
    LoadIndexingStatus,
    LoadAudit,
    LoadNeighborhood,

    // —— 页面动作（由视图按钮发出；具体命令在 app.rs update 里集中分发）——
    HomeCreateProject,
    SearchSubmit,
    AgentAsk,
    AgentSaveReport,
    AgentSaveJob,
    SimulationRun,
    BackupExport,
    BackupImport,
    IndexingBuild,
    JobCancel(String),
    JobRetry(String),
    JobOpenLogs(String),
    JobCloseLogs,
    PromptSelect(String),
    PromptNew,
    PromptSave,
    PromptPreview,
    PromptReset,
    PromptVariableValue(usize, String),
    AiSettingsSave,
    AiSettingsTest(String),
    AiSettingsToggle(String),
    AiSettingsField(AiSettingsField),

    // —— 结果回投（后台或内联完成后写回 store）——
    CommandDone(Box<CommandDone>),

    // —— 对象工作台（Projects 页）——
    SelectProject(String),
    ProjectNameDraftChanged(String),
    ProjectDescriptionDraftChanged(String),
    CreateProject,
    SelectCollection(Collection),
    SelectObject(Option<ObjectRef>),
    NewRecord,
    SaveRecord,
    DeleteRecord,
    ConfirmSuggestion(Box<RelationSuggestion>),
    EditorChanged(EditorField, String),
    EditorNumberChanged(EditorNumberField, f64),
    EditorDirectedToggled(bool),

    // —— 图谱页 ——
    GraphRelationFilterChanged(String),
    GraphEntityFilterChanged(String),
    GraphFocusChanged(Option<String>),
    GraphViewportChanged(GraphViewport),
    GraphViewportReset,
    OpenObjectInWorkbench(ObjectRef),

    // —— 角色成长页 ——
    LoadGrowthWorkspace,
    GrowthCharacterSelected(Option<String>),
    GrowthSourceEventSelected(Option<String>),
    GrowthTraitNameChanged(String),
    GrowthDeltaChanged(String),
    GrowthReasonChanged(String),
    GrowthPreview,
    GrowthSave,
}

/// 一次命令完成后的回投载荷。
#[derive(Debug, Clone)]
pub struct CommandDone {
    /// 发起时记录的 busy tag（用于清理）。
    pub tag: &'static str,
    pub outcome: Result<CommandOutcome, String>,
}

/// 命令结果的类型化载荷。
#[derive(Debug, Clone)]
pub enum CommandOutcome {
    Value(serde_json::Value),
    Projects(Vec<Project>),
    LibrarySnapshot(Box<ameya_core::services::library::LibraryProjectSnapshot>),
    Jobs(Vec<AiJob>),
    CurrentJob(Option<AiJob>),
    JobLogs(Vec<AiJobLog>),
    Prompts(Vec<PromptTemplate>),
    PromptPreview(PromptTemplatePreview),
    RelationPresets(Vec<String>),
    Neighborhood(Box<RelationNeighborhood>),
    ProviderSettings(Vec<AiProviderSettingsView>),
    Diagnostics {
        app_version: String,
        platform: String,
        database: String,
    },
    RagStatus(RagIndexStatus),
    SearchResults(Vec<SearchResult>),
    SemanticSearch(SemanticSearchResponse),
    GrowthWorkspace(Box<CharacterGrowthWorkspaceSnapshot>),
    TraitPreview(CharacterTraitState),
    SimulationReport(SimulationReport),
    AuditConflicts(Vec<LogicConflict>, Vec<RepairSuggestion>),
    AuditRepairs(Vec<RepairSuggestion>),
    ArchiveText(String),
    Notice(String),
}

impl PromptsState {
    /// 选中模板 → 编辑器草稿（含变量值槽位，默认取示例）。
    pub fn fill_draft_from_selected(&mut self) {
        let Some(template) = self
            .templates
            .iter()
            .find(|template| Some(&template.id) == self.selected_id.as_ref())
        else {
            return;
        };
        self.draft_id = Some(template.id.clone());
        self.draft_name = template.name.clone();
        self.draft_purpose = template.purpose.clone();
        self.draft_template = template.template.clone();
        self.variable_values = template
            .variables
            .iter()
            .map(|variable| (variable.name.clone(), variable.example.clone()))
            .collect();
        self.preview = None;
        self.error = None;
        self.notice = None;
    }

    /// 新建模板：清空草稿（保存时由服务端生成 ID）。
    pub fn start_new_draft(&mut self) {
        self.selected_id = None;
        self.draft_id = None;
        self.draft_name = String::new();
        self.draft_purpose = String::new();
        self.draft_template = String::new();
        self.variable_values = Vec::new();
        self.preview = None;
        self.error = None;
        self.notice = None;
    }
}
