//! Auditable, framework-neutral business content descriptors for each native route.
//!
//! The model is deliberately data-only: a hosted Nana view and the deterministic
//! snapshot renderer can consume the same section/field/list/status/empty/action
//! records without reaching the database or inventing a second command contract.

use crate::Route;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageValueKind {
    Identifier,
    Text,
    MultilineText,
    Number,
    Decimal,
    Boolean,
    Timestamp,
    Status,
    Tags,
    EntityReference,
    Json,
}

impl PageValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identifier => "identifier",
            Self::Text => "text",
            Self::MultilineText => "multiline",
            Self::Number => "number",
            Self::Decimal => "decimal",
            Self::Boolean => "boolean",
            Self::Timestamp => "timestamp",
            Self::Status => "status",
            Self::Tags => "tags",
            Self::EntityReference => "entity-reference",
            Self::Json => "json",
        }
    }
}

/// Semantic input widget expected by a hosted renderer for a business field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldControlKind {
    ReadOnly,
    TextInput,
    TextArea,
    NumberInput,
    Toggle,
    DateTime,
    StatusBadge,
    TagInput,
    EntityPicker,
    JsonEditor,
}

impl FieldControlKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "readonly",
            Self::TextInput => "text-input",
            Self::TextArea => "text-area",
            Self::NumberInput => "number-input",
            Self::Toggle => "toggle",
            Self::DateTime => "date-time",
            Self::StatusBadge => "status-badge",
            Self::TagInput => "tag-input",
            Self::EntityPicker => "entity-picker",
            Self::JsonEditor => "json-editor",
        }
    }
}

/// Presentation semantics for a business list rather than a generic table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPresentation {
    Table,
    CompactTable,
    Timeline,
    Cards,
    ResultCards,
    Graph,
    StatusQueue,
    KeyValue,
    ConflictTable,
    Pipeline,
    Transfer,
    Preview,
}

impl ListPresentation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::CompactTable => "compact-table",
            Self::Timeline => "timeline",
            Self::Cards => "cards",
            Self::ResultCards => "result-cards",
            Self::Graph => "graph",
            Self::StatusQueue => "status-queue",
            Self::KeyValue => "key-value",
            Self::ConflictTable => "conflict-table",
            Self::Pipeline => "pipeline",
            Self::Transfer => "transfer",
            Self::Preview => "preview",
        }
    }
}

/// Placement of a route action in the page's interaction hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionPlacement {
    Header,
    Toolbar,
    Primary,
    Secondary,
    Inline,
    EmptyState,
    Footer,
    Inspector,
}

impl ActionPlacement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Toolbar => "toolbar",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Inline => "inline",
            Self::EmptyState => "empty-state",
            Self::Footer => "footer",
            Self::Inspector => "inspector",
        }
    }
}

/// Semantic panel roles used by a route-specific page layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Header,
    Summary,
    Form,
    Table,
    Timeline,
    Audit,
    Graph,
    Results,
    Pipeline,
    Queue,
    Transfer,
    Report,
    Console,
    Settings,
    Help,
    Diagnostics,
    Status,
    Inspector,
}

impl PanelKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Summary => "summary",
            Self::Form => "form",
            Self::Table => "table",
            Self::Timeline => "timeline",
            Self::Audit => "audit",
            Self::Graph => "graph",
            Self::Results => "results",
            Self::Pipeline => "pipeline",
            Self::Queue => "queue",
            Self::Transfer => "transfer",
            Self::Report => "report",
            Self::Console => "console",
            Self::Settings => "settings",
            Self::Help => "help",
            Self::Diagnostics => "diagnostics",
            Self::Status => "status",
            Self::Inspector => "inspector",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageField {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: PageValueKind,
    pub control: FieldControlKind,
    pub source: &'static str,
    pub required: bool,
}

impl PageField {
    pub const fn new(
        id: &'static str,
        label: &'static str,
        kind: PageValueKind,
        source: &'static str,
        required: bool,
    ) -> Self {
        Self {
            id,
            label,
            kind,
            control: control_for_value(kind),
            source,
            required,
        }
    }

    pub const fn with_control(
        id: &'static str,
        label: &'static str,
        kind: PageValueKind,
        control: FieldControlKind,
        source: &'static str,
        required: bool,
    ) -> Self {
        Self {
            id,
            label,
            kind,
            control,
            source,
            required,
        }
    }
}

const fn control_for_value(kind: PageValueKind) -> FieldControlKind {
    match kind {
        PageValueKind::Identifier => FieldControlKind::ReadOnly,
        PageValueKind::Text => FieldControlKind::TextInput,
        PageValueKind::MultilineText => FieldControlKind::TextArea,
        PageValueKind::Number | PageValueKind::Decimal => FieldControlKind::NumberInput,
        PageValueKind::Boolean => FieldControlKind::Toggle,
        PageValueKind::Timestamp => FieldControlKind::DateTime,
        PageValueKind::Status => FieldControlKind::StatusBadge,
        PageValueKind::Tags => FieldControlKind::TagInput,
        PageValueKind::EntityReference => FieldControlKind::EntityPicker,
        PageValueKind::Json => FieldControlKind::JsonEditor,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageStatus {
    pub code: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
}

impl PageStatus {
    pub const fn new(code: &'static str, label: &'static str, detail: &'static str) -> Self {
        Self {
            code,
            label,
            detail,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyState {
    pub title: &'static str,
    pub description: &'static str,
    pub action_id: Option<&'static str>,
}

impl EmptyState {
    pub const fn new(
        title: &'static str,
        description: &'static str,
        action_id: Option<&'static str>,
    ) -> Self {
        Self {
            title,
            description,
            action_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageList {
    pub id: &'static str,
    pub label: &'static str,
    pub item_label: &'static str,
    pub source: &'static str,
    pub presentation: ListPresentation,
    pub columns: Vec<PageField>,
    pub empty: EmptyState,
}

impl PageList {
    pub fn new(
        id: &'static str,
        label: &'static str,
        item_label: &'static str,
        source: &'static str,
        columns: &[PageField],
        empty: EmptyState,
    ) -> Self {
        Self {
            id,
            label,
            item_label,
            source,
            presentation: presentation_for_list(id),
            columns: columns.to_vec(),
            empty,
        }
    }

    pub fn with_presentation(mut self, presentation: ListPresentation) -> Self {
        self.presentation = presentation;
        self
    }
}

fn presentation_for_list(id: &str) -> ListPresentation {
    match id {
        "recent-entries" | "projects" => ListPresentation::CompactTable,
        "characters" | "axioms" | "templates" | "providers" => ListPresentation::Cards,
        "events" | "phases" => ListPresentation::Timeline,
        "facts" | "conflicts" => ListPresentation::ConflictTable,
        "relations" | "neighborhood" => ListPresentation::Graph,
        "results" => ListPresentation::ResultCards,
        "chunks" | "growth-records" => ListPresentation::Pipeline,
        "jobs" => ListPresentation::StatusQueue,
        "archive" => ListPresentation::Transfer,
        "context" => ListPresentation::Preview,
        _ => ListPresentation::Table,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSection {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub status: PageStatus,
    pub fields: Vec<PageField>,
    pub lists: Vec<PageList>,
    pub empty: EmptyState,
}

impl PageSection {
    pub fn new(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        status: PageStatus,
        fields: &[PageField],
        lists: Vec<PageList>,
        empty: EmptyState,
    ) -> Self {
        Self {
            id,
            title,
            description,
            status,
            fields: fields.to_vec(),
            lists,
            empty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageActionKind {
    Navigate,
    Query,
    Mutate,
    Preview,
    Execute,
    Import,
    Export,
    Settings,
}

impl PageActionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Query => "query",
            Self::Mutate => "mutate",
            Self::Preview => "preview",
            Self::Execute => "execute",
            Self::Import => "import",
            Self::Export => "export",
            Self::Settings => "settings",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageAction {
    pub id: &'static str,
    pub label: &'static str,
    pub command: &'static str,
    pub kind: PageActionKind,
    pub placement: ActionPlacement,
    pub description: &'static str,
    pub inputs: Vec<PageField>,
}

impl PageAction {
    pub fn new(
        id: &'static str,
        label: &'static str,
        command: &'static str,
        kind: PageActionKind,
        description: &'static str,
        inputs: &[PageField],
    ) -> Self {
        Self {
            id,
            label,
            command,
            kind,
            placement: placement_for_action(id),
            description,
            inputs: inputs.to_vec(),
        }
    }
}

fn placement_for_action(id: &str) -> ActionPlacement {
    match id {
        "open-project" | "health-check" | "diagnostics" => ActionPlacement::Header,
        "create-project" | "create-event" | "save-growth" | "save-settings" => {
            ActionPlacement::Primary
        }
        "update-event" | "repair-suggestions" | "retry-job" => ActionPlacement::Inline,
        "preview-growth" | "preview-prompt" | "semantic-search" => ActionPlacement::Secondary,
        "export-project" | "import-project" => ActionPlacement::Toolbar,
        "provider-settings" | "load-settings" => ActionPlacement::Inspector,
        "list-prompts" | "library-snapshot" | "keyword-search" | "relation-presets" => {
            ActionPlacement::Toolbar
        }
        _ => ActionPlacement::Toolbar,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagePanel {
    pub id: &'static str,
    pub title: &'static str,
    pub section_id: &'static str,
    pub kind: PanelKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageActionPlacement {
    pub action_id: &'static str,
    pub placement: ActionPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLayout {
    /// Stable route-specific layout identity for hosted renderers and audits.
    pub variant: &'static str,
    pub panels: Vec<PagePanel>,
    pub action_placements: Vec<PageActionPlacement>,
}

impl PageLayout {
    pub fn panel(&self, id: &str) -> Option<&PagePanel> {
        self.panels.iter().find(|panel| panel.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageModel {
    pub route: Route,
    pub summary: &'static str,
    pub layout: PageLayout,
    pub sections: Vec<PageSection>,
    pub actions: Vec<PageAction>,
}

impl PageModel {
    pub fn for_route(route: Route) -> Self {
        model_for(route)
    }

    pub fn action(&self, id: &str) -> Option<&PageAction> {
        self.actions.iter().find(|action| action.id == id)
    }

    pub fn field_count(&self) -> usize {
        self.sections
            .iter()
            .map(|section| {
                section.fields.len()
                    + section
                        .lists
                        .iter()
                        .map(|list| list.columns.len())
                        .sum::<usize>()
            })
            .sum()
    }
}

impl Route {
    /// Return the route's complete business content contract.
    pub fn page_model(self) -> PageModel {
        PageModel::for_route(self)
    }
}

const fn field(
    id: &'static str,
    label: &'static str,
    kind: PageValueKind,
    source: &'static str,
) -> PageField {
    PageField::new(id, label, kind, source, false)
}

const fn required_field(
    id: &'static str,
    label: &'static str,
    kind: PageValueKind,
    source: &'static str,
) -> PageField {
    PageField::new(id, label, kind, source, true)
}

const fn status(code: &'static str, label: &'static str, detail: &'static str) -> PageStatus {
    PageStatus::new(code, label, detail)
}

const fn empty(
    title: &'static str,
    description: &'static str,
    action_id: Option<&'static str>,
) -> EmptyState {
    EmptyState::new(title, description, action_id)
}

fn list(
    id: &'static str,
    label: &'static str,
    item_label: &'static str,
    source: &'static str,
    columns: &[PageField],
    empty_state: EmptyState,
) -> PageList {
    PageList::new(id, label, item_label, source, columns, empty_state)
}

fn section(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    page_status: PageStatus,
    fields: &[PageField],
    lists: Vec<PageList>,
    empty_state: EmptyState,
) -> PageSection {
    PageSection::new(
        id,
        title,
        description,
        page_status,
        fields,
        lists,
        empty_state,
    )
}

fn action(
    id: &'static str,
    label: &'static str,
    command: &'static str,
    kind: PageActionKind,
    description: &'static str,
    inputs: &[PageField],
) -> PageAction {
    PageAction::new(id, label, command, kind, description, inputs)
}

const PROJECT_FIELDS: &[PageField] = &[
    field(
        "projectId",
        "项目 ID",
        PageValueKind::Identifier,
        "project.id",
    ),
    required_field("name", "项目名称", PageValueKind::Text, "project.name"),
    field(
        "description",
        "项目描述",
        PageValueKind::MultilineText,
        "project.description",
    ),
    field(
        "updatedAt",
        "最近更新",
        PageValueKind::Timestamp,
        "project.updatedAt",
    ),
    field(
        "archivedAt",
        "归档时间",
        PageValueKind::Timestamp,
        "project.archivedAt",
    ),
];
const PROJECT_COLUMNS: &[PageField] = &[
    field("id", "项目 ID", PageValueKind::Identifier, "project.id"),
    field("name", "名称", PageValueKind::Text, "project.name"),
    field(
        "description",
        "描述",
        PageValueKind::MultilineText,
        "project.description",
    ),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "project.updatedAt",
    ),
    field(
        "archivedAt",
        "归档状态",
        PageValueKind::Timestamp,
        "project.archivedAt",
    ),
];
const PROJECT_DRAFT: &[PageField] = &[
    required_field(
        "draft.name",
        "项目名称",
        PageValueKind::Text,
        "args.draft.name",
    ),
    field(
        "draft.description",
        "项目描述",
        PageValueKind::MultilineText,
        "args.draft.description",
    ),
];

const ENTRY_COLUMNS: &[PageField] = &[
    field("id", "条目 ID", PageValueKind::Identifier, "entry.id"),
    field("entryType", "类型", PageValueKind::Text, "entry.entryType"),
    field("title", "标题", PageValueKind::Text, "entry.title"),
    field(
        "summary",
        "摘要",
        PageValueKind::MultilineText,
        "entry.summary",
    ),
    field("status", "状态", PageValueKind::Status, "entry.status"),
    field("tags", "标签", PageValueKind::Tags, "entry.tags"),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "entry.updatedAt",
    ),
];
const CHARACTER_COLUMNS: &[PageField] = &[
    field("id", "角色 ID", PageValueKind::Identifier, "character.id"),
    field("name", "姓名", PageValueKind::Text, "character.name"),
    field("aliases", "别名", PageValueKind::Tags, "character.aliases"),
    field(
        "summary",
        "摘要",
        PageValueKind::MultilineText,
        "character.summary",
    ),
    field("faction", "阵营", PageValueKind::Text, "character.faction"),
    field("tags", "标签", PageValueKind::Tags, "character.tags"),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "character.updatedAt",
    ),
];
const EVENT_COLUMNS: &[PageField] = &[
    field("id", "事件 ID", PageValueKind::Identifier, "event.id"),
    field("title", "标题", PageValueKind::Text, "event.title"),
    field("timeLabel", "时间", PageValueKind::Text, "event.timeLabel"),
    field("location", "地点", PageValueKind::Text, "event.location"),
    field(
        "importance",
        "重要度",
        PageValueKind::Number,
        "event.importance",
    ),
    field(
        "outcome",
        "结果",
        PageValueKind::MultilineText,
        "event.outcome",
    ),
    field("tags", "标签", PageValueKind::Tags, "event.tags"),
];
const AXIOM_COLUMNS: &[PageField] = &[
    field("id", "公理 ID", PageValueKind::Identifier, "axiom.id"),
    field("subject", "主体", PageValueKind::Text, "axiom.subject"),
    field("predicate", "谓词", PageValueKind::Text, "axiom.predicate"),
    field("object", "客体", PageValueKind::Text, "axiom.object"),
    field(
        "scopeTime",
        "时间范围",
        PageValueKind::Text,
        "axiom.scopeTime",
    ),
    field(
        "scopeLocation",
        "地点范围",
        PageValueKind::Text,
        "axiom.scopeLocation",
    ),
    field(
        "certainty",
        "确定性",
        PageValueKind::Decimal,
        "axiom.certainty",
    ),
];
const RELATION_COLUMNS: &[PageField] = &[
    field("id", "关系 ID", PageValueKind::Identifier, "relation.id"),
    field(
        "source",
        "来源实体",
        PageValueKind::EntityReference,
        "relation.source",
    ),
    field(
        "target",
        "目标实体",
        PageValueKind::EntityReference,
        "relation.target",
    ),
    field(
        "relationType",
        "关系类型",
        PageValueKind::Text,
        "relation.relationType",
    ),
    field(
        "description",
        "关系描述",
        PageValueKind::MultilineText,
        "relation.description",
    ),
    field(
        "confidence",
        "置信度",
        PageValueKind::Decimal,
        "relation.confidence",
    ),
    field(
        "directed",
        "有向",
        PageValueKind::Boolean,
        "relation.directed",
    ),
];
const SEARCH_COLUMNS: &[PageField] = &[
    field(
        "entityType",
        "实体类型",
        PageValueKind::Text,
        "result.entityType",
    ),
    field(
        "entityId",
        "实体 ID",
        PageValueKind::Identifier,
        "result.entityId",
    ),
    field("title", "标题", PageValueKind::Text, "result.title"),
    field(
        "snippet",
        "摘要片段",
        PageValueKind::MultilineText,
        "result.snippet",
    ),
    field("score", "匹配分数", PageValueKind::Decimal, "result.score"),
];
const CHUNK_COLUMNS: &[PageField] = &[
    field("id", "分块 ID", PageValueKind::Identifier, "chunk.id"),
    field(
        "sourceType",
        "来源类型",
        PageValueKind::Text,
        "chunk.sourceType",
    ),
    field(
        "sourceId",
        "来源 ID",
        PageValueKind::Identifier,
        "chunk.sourceId",
    ),
    field("ordinal", "序号", PageValueKind::Number, "chunk.ordinal"),
    field(
        "estimatedTokens",
        "估算 token",
        PageValueKind::Number,
        "chunk.estimatedTokens",
    ),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "chunk.updatedAt",
    ),
];
const JOB_COLUMNS: &[PageField] = &[
    field("id", "任务 ID", PageValueKind::Identifier, "job.id"),
    field(
        "providerKind",
        "提供方",
        PageValueKind::Text,
        "job.providerKind",
    ),
    field("jobType", "任务类型", PageValueKind::Text, "job.jobType"),
    field("status", "状态", PageValueKind::Status, "job.status"),
    field(
        "inputSummary",
        "输入摘要",
        PageValueKind::MultilineText,
        "job.inputSummary",
    ),
    field(
        "errorMessage",
        "错误",
        PageValueKind::MultilineText,
        "job.errorMessage",
    ),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "job.updatedAt",
    ),
];
const PROMPT_COLUMNS: &[PageField] = &[
    field("id", "模板 ID", PageValueKind::Identifier, "template.id"),
    field("name", "名称", PageValueKind::Text, "template.name"),
    field("purpose", "用途", PageValueKind::Text, "template.purpose"),
    field(
        "builtIn",
        "内置",
        PageValueKind::Boolean,
        "template.builtIn",
    ),
    field(
        "variables",
        "变量",
        PageValueKind::Json,
        "template.variables",
    ),
    field(
        "updatedAt",
        "更新时间",
        PageValueKind::Timestamp,
        "template.updatedAt",
    ),
];

fn layout_for(route: Route, sections: &[PageSection], actions: &[PageAction]) -> PageLayout {
    let variant = match route {
        Route::Home => "dashboard",
        Route::Projects => "project-workspace",
        Route::Search => "search-workbench",
        Route::Graph => "relation-graph",
        Route::Timeline => "event-timeline",
        Route::Audit => "audit-review",
        Route::Indexing => "indexing-pipeline",
        Route::Jobs => "job-queue",
        Route::PromptTemplates => "prompt-library",
        Route::Settings => "settings-form",
        Route::Agent => "agent-console",
        Route::Backup => "backup-transfer",
        Route::Simulation => "simulation-report",
        Route::Growth => "growth-workspace",
        Route::Help => "command-help",
        Route::Diagnostics => "diagnostics-console",
    };
    let panels = sections
        .iter()
        .enumerate()
        .map(|(index, section)| PagePanel {
            id: section.id,
            title: section.title,
            section_id: section.id,
            kind: panel_kind(route, index, section),
        })
        .collect();
    let action_placements = actions
        .iter()
        .map(|action| PageActionPlacement {
            action_id: action.id,
            placement: action.placement,
        })
        .collect();
    PageLayout {
        variant,
        panels,
        action_placements,
    }
}

fn panel_kind(route: Route, index: usize, section: &PageSection) -> PanelKind {
    match route {
        Route::Home => {
            if index == 0 {
                PanelKind::Summary
            } else {
                PanelKind::Status
            }
        }
        Route::Projects => {
            if section.lists.is_empty() {
                PanelKind::Inspector
            } else {
                PanelKind::Table
            }
        }
        Route::Search => PanelKind::Results,
        Route::Graph => PanelKind::Graph,
        Route::Timeline => PanelKind::Timeline,
        Route::Audit => PanelKind::Audit,
        Route::Indexing => PanelKind::Pipeline,
        Route::Jobs => PanelKind::Queue,
        Route::PromptTemplates => PanelKind::Form,
        Route::Settings => PanelKind::Settings,
        Route::Agent => PanelKind::Console,
        Route::Backup => PanelKind::Transfer,
        Route::Simulation => PanelKind::Report,
        Route::Growth => PanelKind::Inspector,
        Route::Help => PanelKind::Help,
        Route::Diagnostics => PanelKind::Diagnostics,
    }
}

fn model_for(route: Route) -> PageModel {
    let (summary, sections, actions) = match route {
        Route::Home => (
            "项目工作台：选择项目后查看资料库、角色和事件的最近变更。",
            vec![
                section(
                    "project-overview",
                    "当前项目",
                    "项目身份、描述和最近更新时间。",
                    status("ready", "可用", "项目列表可读取；未选项目时显示空态。"),
                    PROJECT_FIELDS,
                    vec![list(
                        "recent-entries",
                        "最近更新条目",
                        "条目",
                        "project.entries",
                        ENTRY_COLUMNS,
                        empty(
                            "暂无条目",
                            "创建第一条资料库条目后，它会出现在这里。",
                            Some("open-project"),
                        ),
                    )],
                    empty(
                        "尚未选择项目",
                        "打开一个项目以查看业务内容。",
                        Some("open-project"),
                    ),
                ),
                section(
                    "connection-health",
                    "连接状态",
                    "本地数据库和 native 命令边界的运行信息。",
                    status("local", "本地模式", "数据通过 NativeCommandHost 访问。"),
                    &[
                        field(
                            "appVersion",
                            "应用版本",
                            PageValueKind::Text,
                            "health.appVersion",
                        ),
                        field("platform", "平台", PageValueKind::Text, "health.platform"),
                        field(
                            "appDataDir",
                            "数据目录",
                            PageValueKind::Text,
                            "health.appDataDir",
                        ),
                    ],
                    vec![],
                    empty(
                        "尚未检查连接",
                        "运行健康检查以读取 native 环境信息。",
                        Some("run-check"),
                    ),
                ),
            ],
            vec![
                action(
                    "open-project",
                    "打开项目",
                    "project_list",
                    PageActionKind::Query,
                    "读取未归档项目列表。",
                    &[],
                ),
                action(
                    "run-check",
                    "运行校验",
                    "health_check",
                    PageActionKind::Query,
                    "读取应用版本、平台和数据目录。",
                    &[],
                ),
            ],
        ),
        Route::Projects => (
            "资料库项目管理：创建、查看和归档工作项目。",
            vec![section(
                "project-list",
                "项目列表",
                "按更新时间倒序显示未归档项目。",
                status("ready", "可用", "project_list 返回 active projects。"),
                PROJECT_FIELDS,
                vec![
                    list(
                        "projects",
                        "项目",
                        "项目",
                        "project_list",
                        PROJECT_COLUMNS,
                        empty(
                            "还没有项目",
                            "创建一个项目开始整理世界观资料。",
                            Some("create-project"),
                        ),
                    ),
                    list(
                        "characters",
                        "角色",
                        "角色",
                        "project.characters",
                        CHARACTER_COLUMNS,
                        empty("暂无角色", "为项目建立第一个角色。", None),
                    ),
                    list(
                        "axioms",
                        "世界规则",
                        "公理",
                        "project.axioms",
                        AXIOM_COLUMNS,
                        empty("暂无公理", "为项目记录第一条世界规则。", None),
                    ),
                ],
                empty(
                    "还没有项目",
                    "创建一个项目开始整理世界观资料。",
                    Some("create-project"),
                ),
            )],
            vec![
                action(
                    "create-project",
                    "新建项目",
                    "project_create",
                    PageActionKind::Mutate,
                    "使用 draft.name 和 draft.description 创建项目。",
                    PROJECT_DRAFT,
                ),
                action(
                    "library-snapshot",
                    "资料库快照",
                    "library_project_snapshot",
                    PageActionKind::Query,
                    "读取项目及其条目、角色、事件等关联数据。",
                    &[required_field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    )],
                ),
            ],
        ),
        Route::Growth => (
            "角色成长工作台：按事件记录角色特质变化并保留来源。",
            vec![section(
                "trait-state",
                "角色特质状态",
                "状态由 CharacterTraitDeltaRecord 按角色聚合，数值限制在 -1 到 1。",
                status(
                    "ready",
                    "可计算",
                    "已有记录会生成每个角色的特质值和来源链。",
                ),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    ),
                    field(
                        "characterId",
                        "角色 ID",
                        PageValueKind::Identifier,
                        "record.characterId",
                    ),
                    field(
                        "traitName",
                        "特质名称",
                        PageValueKind::Text,
                        "record.traitName",
                    ),
                    field("delta", "变化量", PageValueKind::Decimal, "record.delta"),
                    field(
                        "sourceEventId",
                        "来源事件",
                        PageValueKind::Identifier,
                        "record.sourceEventId",
                    ),
                    field(
                        "reason",
                        "变化原因",
                        PageValueKind::MultilineText,
                        "record.reason",
                    ),
                ],
                vec![list(
                    "growth-records",
                    "成长记录",
                    "记录",
                    "character_growth_workspace.records",
                    &[
                        field("id", "记录 ID", PageValueKind::Identifier, "record.id"),
                        field(
                            "characterId",
                            "角色 ID",
                            PageValueKind::Identifier,
                            "record.characterId",
                        ),
                        field("traitName", "特质", PageValueKind::Text, "record.traitName"),
                        field("delta", "变化量", PageValueKind::Decimal, "record.delta"),
                        field(
                            "reason",
                            "原因",
                            PageValueKind::MultilineText,
                            "record.reason",
                        ),
                    ],
                    empty(
                        "暂无成长记录",
                        "从一个已存在的角色和事件创建第一条变化记录。",
                        Some("save-growth"),
                    ),
                )],
                empty(
                    "暂无角色状态",
                    "读取成长工作区后，按角色显示聚合特质状态。",
                    Some("preview-growth"),
                ),
            )],
            vec![
                action(
                    "preview-growth",
                    "预览成长",
                    "character_growth_preview_trait_delta",
                    PageActionKind::Preview,
                    "在内存中应用一个 trait delta，不写入数据库。",
                    &[
                        required_field("state", "当前状态", PageValueKind::Json, "args.state"),
                        required_field("delta", "变化记录", PageValueKind::Json, "args.delta"),
                    ],
                ),
                action(
                    "save-growth",
                    "保存成长",
                    "character_growth_create_record",
                    PageActionKind::Mutate,
                    "校验项目、角色和来源事件均为 active 后创建记录。",
                    &[required_field(
                        "draft",
                        "成长记录草稿",
                        PageValueKind::Json,
                        "args.draft",
                    )],
                ),
            ],
        ),
        Route::Timeline => (
            "事件时间线：以时间排序浏览事件、地点、重要度和结果。",
            vec![section(
                "event-timeline",
                "事件时间线",
                "事件按 sortKey 升序、更新时间倒序排列。",
                status("ready", "可用", "事件支持参与者和标签等业务字段。"),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "event.projectId",
                    ),
                    field("title", "标题", PageValueKind::Text, "event.title"),
                    field(
                        "timeLabel",
                        "时间标签",
                        PageValueKind::Text,
                        "event.timeLabel",
                    ),
                    field(
                        "startLabel",
                        "开始",
                        PageValueKind::Text,
                        "event.startLabel",
                    ),
                    field("endLabel", "结束", PageValueKind::Text, "event.endLabel"),
                    field("location", "地点", PageValueKind::Text, "event.location"),
                    field(
                        "importance",
                        "重要度",
                        PageValueKind::Number,
                        "event.importance",
                    ),
                    field(
                        "outcome",
                        "结果",
                        PageValueKind::MultilineText,
                        "event.outcome",
                    ),
                    field("tags", "标签", PageValueKind::Tags, "event.tags"),
                ],
                vec![list(
                    "events",
                    "事件",
                    "事件",
                    "project.events",
                    EVENT_COLUMNS,
                    empty("暂无事件", "新建事件以建立时间线。", Some("create-event")),
                )],
                empty("暂无事件", "新建事件以建立时间线。", Some("create-event")),
            )],
            vec![
                action(
                    "create-event",
                    "新建事件",
                    "library_create_event",
                    PageActionKind::Mutate,
                    "按 EventDraft 和 participants 创建事件。",
                    &[
                        required_field("draft", "事件草稿", PageValueKind::Json, "args.draft"),
                        field(
                            "participants",
                            "参与者",
                            PageValueKind::Json,
                            "args.participants",
                        ),
                    ],
                ),
                action(
                    "update-event",
                    "更新事件",
                    "library_update_event",
                    PageActionKind::Mutate,
                    "使用 id、EventDraft 和 participants 更新事件。",
                    &[
                        required_field("id", "事件 ID", PageValueKind::Identifier, "args.id"),
                        required_field("draft", "事件草稿", PageValueKind::Json, "args.draft"),
                        field(
                            "participants",
                            "参与者",
                            PageValueKind::Json,
                            "args.participants",
                        ),
                    ],
                ),
            ],
        ),
        Route::Audit => (
            "规则审计：检查同一范围内互斥事实并生成可追溯修复建议。",
            vec![section(
                "logic-audit",
                "事实冲突",
                "冲突键为 subject、predicate、scopeTime、scopeLocation；object 不同即构成冲突。",
                status(
                    "deterministic",
                    "可复现",
                    "审计在本地纯函数中执行，不修改数据。",
                ),
                &[
                    field("id", "事实 ID", PageValueKind::Identifier, "fact.id"),
                    field("subject", "主体", PageValueKind::Text, "fact.subject"),
                    field("predicate", "谓词", PageValueKind::Text, "fact.predicate"),
                    field("object", "客体", PageValueKind::Text, "fact.object"),
                    field(
                        "scopeTime",
                        "时间范围",
                        PageValueKind::Text,
                        "fact.scopeTime",
                    ),
                    field(
                        "scopeLocation",
                        "地点范围",
                        PageValueKind::Text,
                        "fact.scopeLocation",
                    ),
                ],
                vec![
                    list(
                        "facts",
                        "事实",
                        "事实",
                        "args.facts",
                        &[
                            field("id", "事实 ID", PageValueKind::Identifier, "fact.id"),
                            field("subject", "主体", PageValueKind::Text, "fact.subject"),
                            field("predicate", "谓词", PageValueKind::Text, "fact.predicate"),
                            field("object", "客体", PageValueKind::Text, "fact.object"),
                            field(
                                "scopeTime",
                                "时间范围",
                                PageValueKind::Text,
                                "fact.scopeTime",
                            ),
                            field(
                                "scopeLocation",
                                "地点范围",
                                PageValueKind::Text,
                                "fact.scopeLocation",
                            ),
                        ],
                        empty("暂无事实", "提供事实后运行审计。", Some("audit-facts")),
                    ),
                    list(
                        "conflicts",
                        "冲突",
                        "冲突",
                        "audit.conflicts",
                        &[
                            field(
                                "conflictType",
                                "冲突类型",
                                PageValueKind::Text,
                                "conflict.conflictType",
                            ),
                            field(
                                "factIds",
                                "事实 ID",
                                PageValueKind::Json,
                                "conflict.factIds",
                            ),
                            field(
                                "message",
                                "说明",
                                PageValueKind::MultilineText,
                                "conflict.message",
                            ),
                        ],
                        empty("未发现冲突", "当前事实集合通过互斥规则检查。", None),
                    ),
                ],
                empty(
                    "暂无审计结果",
                    "运行审计后显示事实冲突和修复建议。",
                    Some("audit-facts"),
                ),
            )],
            vec![
                action(
                    "audit-facts",
                    "运行审计",
                    "logic_audit_facts",
                    PageActionKind::Execute,
                    "检测事实集合中的互斥公理。",
                    &[required_field(
                        "facts",
                        "事实列表",
                        PageValueKind::Json,
                        "args.facts",
                    )],
                ),
                action(
                    "repair-suggestions",
                    "修复建议",
                    "logic_repair_suggestions",
                    PageActionKind::Preview,
                    "根据单个 LogicConflict 给出候选修复。",
                    &[required_field(
                        "conflict",
                        "冲突",
                        PageValueKind::Json,
                        "args.conflict",
                    )],
                ),
            ],
        ),
        Route::Graph => (
            "关系图谱：浏览实体邻域、关系类型预设和关系置信度。",
            vec![section(
                "relation-neighborhood",
                "关系邻域",
                "以 EntityRef 为中心按 depth 读取项目关系。",
                status(
                    "ready",
                    "可查询",
                    "支持 backlinks 和项目邻域两种关系读取方式。",
                ),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    ),
                    field(
                        "target",
                        "中心实体",
                        PageValueKind::EntityReference,
                        "args.target",
                    ),
                    field("depth", "深度", PageValueKind::Number, "args.depth"),
                ],
                vec![list(
                    "relations",
                    "关系",
                    "关系",
                    "relation_neighborhood.relations",
                    RELATION_COLUMNS,
                    empty(
                        "暂无关系",
                        "为实体创建第一条关系。",
                        Some("relation-neighborhood"),
                    ),
                )],
                empty(
                    "未选择实体",
                    "选择一个实体后加载关系邻域。",
                    Some("relation-neighborhood"),
                ),
            )],
            vec![
                action(
                    "relation-neighborhood",
                    "关系邻域",
                    "library_relation_neighborhood",
                    PageActionKind::Query,
                    "读取指定项目和实体的关系邻域。",
                    &[
                        required_field(
                            "projectId",
                            "项目 ID",
                            PageValueKind::Identifier,
                            "args.projectId",
                        ),
                        required_field(
                            "target",
                            "中心实体",
                            PageValueKind::EntityReference,
                            "args.target",
                        ),
                        field("depth", "深度", PageValueKind::Number, "args.depth"),
                    ],
                ),
                action(
                    "relation-presets",
                    "关系预设",
                    "library_relation_type_presets",
                    PageActionKind::Query,
                    "读取内置关系类型及其方向性建议。",
                    &[],
                ),
            ],
        ),
        Route::Search => (
            "分析与检索：关键词搜索实体并在语义不可用时明确降级。",
            vec![section(
                "search-results",
                "检索结果",
                "覆盖 entry、character、event、axiom；结果按 score 降序。",
                status(
                    "degraded-capable",
                    "可用",
                    "semantic_search 可能返回关键词回退状态。",
                ),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.filter.projectId",
                    ),
                    field("query", "关键词", PageValueKind::Text, "args.filter.query"),
                    field(
                        "entityTypes",
                        "实体类型",
                        PageValueKind::Tags,
                        "args.filter.entityTypes",
                    ),
                    field(
                        "status",
                        "语义状态",
                        PageValueKind::Status,
                        "response.status",
                    ),
                    field(
                        "message",
                        "状态说明",
                        PageValueKind::Text,
                        "response.message",
                    ),
                    field("model", "模型", PageValueKind::Text, "response.model"),
                ],
                vec![list(
                    "results",
                    "结果",
                    "结果",
                    "response.items",
                    SEARCH_COLUMNS,
                    empty(
                        "没有匹配结果",
                        "调整关键词或实体类型后重新检索。",
                        Some("keyword-search"),
                    ),
                )],
                empty(
                    "输入关键词开始",
                    "查询会在当前项目的四类实体中匹配。",
                    Some("keyword-search"),
                ),
            )],
            vec![
                action(
                    "keyword-search",
                    "关键词检索",
                    "search_entities",
                    PageActionKind::Query,
                    "按项目、关键词和实体类型执行本地检索。",
                    &[required_field(
                        "filter",
                        "检索过滤器",
                        PageValueKind::Json,
                        "args.filter",
                    )],
                ),
                action(
                    "semantic-search",
                    "语义检索",
                    "search_semantic",
                    PageActionKind::Query,
                    "执行语义请求；无 embedding provider 时返回 degraded。",
                    &[required_field(
                        "request",
                        "语义请求",
                        PageValueKind::Json,
                        "args.request",
                    )],
                ),
            ],
        ),
        Route::Indexing => (
            "索引：将资料库文本切分为文档块并查看 embedding 覆盖情况。",
            vec![section(
                "rag-index",
                "RAG 索引",
                "索引来源包含 entry、character、event、axiom。",
                status(
                    "ready",
                    "可检查",
                    "状态区分 chunk、embedding、缺失和过期数量。",
                ),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    ),
                    field(
                        "maxChars",
                        "分块上限",
                        PageValueKind::Number,
                        "args.maxChars",
                    ),
                    field(
                        "chunkCount",
                        "分块数",
                        PageValueKind::Number,
                        "status.chunkCount",
                    ),
                    field(
                        "embeddingCount",
                        "向量数",
                        PageValueKind::Number,
                        "status.embeddingCount",
                    ),
                    field(
                        "missingEmbeddingCount",
                        "缺失向量",
                        PageValueKind::Number,
                        "status.missingEmbeddingCount",
                    ),
                    field(
                        "staleEmbeddingCount",
                        "过期向量",
                        PageValueKind::Number,
                        "status.staleEmbeddingCount",
                    ),
                    field("model", "模型", PageValueKind::Text, "status.model"),
                    field("message", "说明", PageValueKind::Text, "status.message"),
                ],
                vec![list(
                    "chunks",
                    "文档块",
                    "文档块",
                    "rag_index_chunks",
                    CHUNK_COLUMNS,
                    empty(
                        "暂无索引分块",
                        "运行索引文本以生成文档块。",
                        Some("index-chunks"),
                    ),
                )],
                empty(
                    "索引尚未建立",
                    "为项目生成文本分块后才能进行向量检索。",
                    Some("index-chunks"),
                ),
            )],
            vec![
                action(
                    "index-chunks",
                    "索引文本",
                    "rag_index_chunks",
                    PageActionKind::Execute,
                    "按 maxChars 重建项目文本分块。",
                    &[
                        required_field(
                            "projectId",
                            "项目 ID",
                            PageValueKind::Identifier,
                            "args.projectId",
                        ),
                        field(
                            "maxChars",
                            "分块上限",
                            PageValueKind::Number,
                            "args.maxChars",
                        ),
                    ],
                ),
                action(
                    "index-embeddings",
                    "生成向量",
                    "rag_index_embeddings",
                    PageActionKind::Execute,
                    "生成索引状态所需的 embedding（需配置 provider）。",
                    &[
                        required_field(
                            "projectId",
                            "项目 ID",
                            PageValueKind::Identifier,
                            "args.projectId",
                        ),
                        field(
                            "maxChars",
                            "分块上限",
                            PageValueKind::Number,
                            "args.maxChars",
                        ),
                    ],
                ),
            ],
        ),
        Route::Backup => (
            "备份：以版本化 ProjectArchive 导出或导入完整项目资料。",
            vec![section(
                "project-archive",
                "项目归档",
                "归档包含项目、条目、角色、事件、公理和关系。",
                status("versioned", "版本化", "当前导出 archive.version 为 2。"),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    ),
                    field(
                        "version",
                        "归档版本",
                        PageValueKind::Number,
                        "archive.version",
                    ),
                    field("project", "项目", PageValueKind::Json, "archive.project"),
                    field("entries", "条目", PageValueKind::Json, "archive.entries"),
                    field(
                        "characters",
                        "角色",
                        PageValueKind::Json,
                        "archive.characters",
                    ),
                    field("events", "事件", PageValueKind::Json, "archive.events"),
                    field("axioms", "公理", PageValueKind::Json, "archive.axioms"),
                    field(
                        "relations",
                        "关系",
                        PageValueKind::Json,
                        "archive.relations",
                    ),
                ],
                vec![],
                empty(
                    "暂无归档",
                    "选择项目后导出 ProjectArchive；导入会创建带“副本”后缀的新项目。",
                    Some("export-project"),
                ),
            )],
            vec![
                action(
                    "export-project",
                    "导出项目",
                    "archive_export_project",
                    PageActionKind::Export,
                    "导出版本化 ProjectArchive。",
                    &[required_field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    )],
                ),
                action(
                    "import-project",
                    "导入项目",
                    "archive_import_project",
                    PageActionKind::Import,
                    "导入 ProjectArchive 并创建项目副本。",
                    &[required_field(
                        "archive",
                        "项目归档",
                        PageValueKind::Json,
                        "args.archive",
                    )],
                ),
            ],
        ),
        Route::Simulation => (
            "模拟：输入场景和引用实体，查看短期、中期阶段与风险。",
            vec![section(
                "simulation-report",
                "模拟报告",
                "模拟输出固定包含短期、中期阶段和资源来源风险提示。",
                status(
                    "deterministic",
                    "可执行",
                    "当前模拟服务为纯函数，不修改数据库。",
                ),
                &[
                    field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "report.projectId",
                    ),
                    field(
                        "scenario",
                        "场景",
                        PageValueKind::MultilineText,
                        "report.scenario",
                    ),
                    field(
                        "referencedEntities",
                        "引用实体",
                        PageValueKind::Json,
                        "report.referencedEntities",
                    ),
                    field("phases", "阶段", PageValueKind::Json, "report.phases"),
                    field("risks", "风险", PageValueKind::Json, "report.risks"),
                ],
                vec![list(
                    "phases",
                    "推演阶段",
                    "阶段",
                    "report.phases",
                    &[
                        field("label", "阶段", PageValueKind::Text, "phase.label"),
                        field(
                            "summary",
                            "摘要",
                            PageValueKind::MultilineText,
                            "phase.summary",
                        ),
                    ],
                    empty(
                        "暂无模拟阶段",
                        "运行模拟后显示阶段推演。",
                        Some("run-simulation"),
                    ),
                )],
                empty(
                    "尚未运行模拟",
                    "描述一个场景并引用相关实体。",
                    Some("run-simulation"),
                ),
            )],
            vec![action(
                "run-simulation",
                "运行模拟",
                "simulation_run",
                PageActionKind::Execute,
                "根据项目、场景和引用实体生成 SimulationReport。",
                &[
                    required_field(
                        "projectId",
                        "项目 ID",
                        PageValueKind::Identifier,
                        "args.projectId",
                    ),
                    required_field(
                        "scenario",
                        "场景",
                        PageValueKind::MultilineText,
                        "args.scenario",
                    ),
                    field(
                        "referencedEntities",
                        "引用实体",
                        PageValueKind::Json,
                        "args.referencedEntities",
                    ),
                ],
            )],
        ),
        Route::Agent => (
            "智能体：选择已配置 provider，提交请求并跟踪异步任务。",
            vec![section(
                "agent-request",
                "智能体请求",
                "provider 设置和密钥不在页面模型中暴露明文，仅通过 native secret store 管理。",
                status(
                    "provider-required",
                    "需要配置",
                    "未配置 provider 时 ai_agent_ask 会返回明确错误。",
                ),
                &[
                    field(
                        "providerKind",
                        "提供方",
                        PageValueKind::Text,
                        "job.providerKind",
                    ),
                    field("jobType", "任务类型", PageValueKind::Text, "job.jobType"),
                    field(
                        "inputSummary",
                        "输入摘要",
                        PageValueKind::MultilineText,
                        "job.inputSummary",
                    ),
                    field(
                        "outputText",
                        "输出",
                        PageValueKind::MultilineText,
                        "job.outputText",
                    ),
                    field(
                        "errorMessage",
                        "错误",
                        PageValueKind::MultilineText,
                        "job.errorMessage",
                    ),
                ],
                vec![],
                empty(
                    "尚未提交请求",
                    "配置 provider 后提交一个 agent 请求。",
                    Some("ask-agent"),
                ),
            )],
            vec![
                action(
                    "ask-agent",
                    "询问智能体",
                    "ai_agent_ask",
                    PageActionKind::Execute,
                    "调用明确配置的 AI provider；不会隐式读取密钥。",
                    &[required_field(
                        "request",
                        "请求",
                        PageValueKind::Json,
                        "args.request",
                    )],
                ),
                action(
                    "provider-settings",
                    "提供方设置",
                    "ai_load_provider_settings",
                    PageActionKind::Settings,
                    "读取 provider 设置视图；密钥仅返回 hasApiKey 和 preview。",
                    &[],
                ),
            ],
        ),
        Route::Settings => (
            "设置：查看 provider 能力并保存不含明文密钥的配置变更。",
            vec![section(
                "provider-settings",
                "AI 提供方",
                "支持 OpenAI-compatible、Codex CLI 和 Claude CLI。",
                status(
                    "secret-store",
                    "安全存储",
                    "API key 由 native secret store 管理。",
                ),
                &[
                    field("kind", "类型", PageValueKind::Text, "settings.kind"),
                    field(
                        "baseUrl",
                        "服务地址",
                        PageValueKind::Text,
                        "settings.baseUrl",
                    ),
                    field(
                        "apiKeyPreview",
                        "密钥预览",
                        PageValueKind::Text,
                        "settings.apiKeyPreview",
                    ),
                    field(
                        "hasApiKey",
                        "已配置密钥",
                        PageValueKind::Boolean,
                        "settings.hasApiKey",
                    ),
                    field(
                        "chatModel",
                        "聊天模型",
                        PageValueKind::Text,
                        "settings.chatModel",
                    ),
                    field(
                        "embeddingModel",
                        "向量模型",
                        PageValueKind::Text,
                        "settings.embeddingModel",
                    ),
                    field(
                        "commandTemplate",
                        "命令模板",
                        PageValueKind::Text,
                        "settings.commandTemplate",
                    ),
                    field(
                        "enabled",
                        "启用",
                        PageValueKind::Boolean,
                        "settings.enabled",
                    ),
                ],
                vec![list(
                    "providers",
                    "提供方",
                    "提供方",
                    "ai_load_provider_settings",
                    &[
                        field("kind", "类型", PageValueKind::Text, "settings.kind"),
                        field(
                            "enabled",
                            "启用",
                            PageValueKind::Boolean,
                            "settings.enabled",
                        ),
                        field(
                            "hasApiKey",
                            "已配置密钥",
                            PageValueKind::Boolean,
                            "settings.hasApiKey",
                        ),
                        field(
                            "chatModel",
                            "聊天模型",
                            PageValueKind::Text,
                            "settings.chatModel",
                        ),
                        field(
                            "embeddingModel",
                            "向量模型",
                            PageValueKind::Text,
                            "settings.embeddingModel",
                        ),
                    ],
                    empty(
                        "暂无提供方",
                        "读取默认 provider 或保存自定义配置。",
                        Some("load-settings"),
                    ),
                )],
                empty(
                    "尚未读取设置",
                    "读取 provider 配置以开始设置。",
                    Some("load-settings"),
                ),
            )],
            vec![
                action(
                    "load-settings",
                    "读取设置",
                    "ai_load_provider_settings",
                    PageActionKind::Settings,
                    "读取 provider 设置视图，不返回明文密钥。",
                    &[],
                ),
                action(
                    "save-settings",
                    "保存设置",
                    "ai_save_provider_settings",
                    PageActionKind::Settings,
                    "通过 native secret store 保存 provider 配置。",
                    &[required_field(
                        "settings",
                        "设置",
                        PageValueKind::Json,
                        "args.settings",
                    )],
                ),
            ],
        ),
        Route::Help => (
            "帮助：查看本地 native 命令边界、数据来源和健康检查结果。",
            vec![section(
                "help-contract",
                "命令契约",
                "每个业务 action 都映射到 NativeCommandHost 的稳定命令名。",
                status(
                    "documented",
                    "已记录",
                    "页面模型中的 command 可与 command_names 逐一审计。",
                ),
                &[
                    field(
                        "command",
                        "命令名",
                        PageValueKind::Text,
                        "route.action.command",
                    ),
                    field(
                        "description",
                        "用途",
                        PageValueKind::MultilineText,
                        "route.action.description",
                    ),
                    field("kind", "动作类型", PageValueKind::Text, "route.action.kind"),
                ],
                vec![],
                empty("暂无契约信息", "当前路由没有可用命令。", None),
            )],
            vec![action(
                "health-check",
                "检查连接",
                "health_check",
                PageActionKind::Query,
                "检查应用版本、平台和数据目录。",
                &[],
            )],
        ),
        Route::Diagnostics => (
            "诊断：生成当前 native 数据目录和运行就绪状态摘要。",
            vec![section(
                "diagnostics-summary",
                "诊断摘要",
                "诊断结果用于定位环境问题，不包含业务正文或 secret。",
                status("safe", "只读", "diagnostics_summary 不修改数据库。"),
                &[
                    field("status", "状态", PageValueKind::Status, "summary.status"),
                    field(
                        "dataDir",
                        "数据目录",
                        PageValueKind::Text,
                        "summary.dataDir",
                    ),
                ],
                vec![],
                empty(
                    "尚未生成诊断",
                    "生成摘要以确认 native 数据目录和状态。",
                    Some("diagnostics"),
                ),
            )],
            vec![action(
                "diagnostics",
                "生成诊断",
                "diagnostics_summary",
                PageActionKind::Query,
                "返回 ready 状态和 native dataDir。",
                &[],
            )],
        ),
        Route::Jobs => (
            "任务：查看 AI 作业状态、日志，并对失败任务执行重试。",
            vec![section(
                "job-queue",
                "AI 任务队列",
                "任务状态包括 queued、running、succeeded、failed、cancelled。",
                status(
                    "observable",
                    "可观测",
                    "job_list 返回完整任务列表，job_current 返回当前 running 任务。",
                ),
                &[
                    field("id", "任务 ID", PageValueKind::Identifier, "job.id"),
                    field("status", "状态", PageValueKind::Status, "job.status"),
                    field(
                        "startedAt",
                        "开始时间",
                        PageValueKind::Timestamp,
                        "job.startedAt",
                    ),
                    field(
                        "finishedAt",
                        "完成时间",
                        PageValueKind::Timestamp,
                        "job.finishedAt",
                    ),
                    field(
                        "cancelRequestedAt",
                        "取消请求",
                        PageValueKind::Timestamp,
                        "job.cancelRequestedAt",
                    ),
                    field(
                        "retryOfJobId",
                        "重试来源",
                        PageValueKind::Identifier,
                        "job.retryOfJobId",
                    ),
                ],
                vec![list(
                    "jobs",
                    "任务",
                    "任务",
                    "job_list",
                    JOB_COLUMNS,
                    empty("暂无任务", "提交一个智能体请求后，任务会出现在这里。", None),
                )],
                empty(
                    "暂无任务",
                    "当前没有 queued 或历史 AI 任务。",
                    Some("list-jobs"),
                ),
            )],
            vec![
                action(
                    "list-jobs",
                    "任务列表",
                    "job_list",
                    PageActionKind::Query,
                    "读取 AI 任务队列。",
                    &[],
                ),
                action(
                    "retry-job",
                    "重试任务",
                    "job_retry",
                    PageActionKind::Mutate,
                    "从指定 jobId 创建同输入的新 queued 任务。",
                    &[required_field(
                        "jobId",
                        "任务 ID",
                        PageValueKind::Identifier,
                        "args.jobId",
                    )],
                ),
            ],
        ),
        Route::PromptTemplates => (
            "提示词模板：管理内置和自定义模板，预览变量缺失情况。",
            vec![section(
                "prompt-library",
                "模板库",
                "模板包含用途、正文、变量和 builtIn 标识。",
                status("ready", "可用", "内置模板由服务启动时补齐。"),
                &[
                    field("id", "模板 ID", PageValueKind::Identifier, "template.id"),
                    field("name", "名称", PageValueKind::Text, "template.name"),
                    field("purpose", "用途", PageValueKind::Text, "template.purpose"),
                    field(
                        "template",
                        "模板正文",
                        PageValueKind::MultilineText,
                        "template.template",
                    ),
                    field(
                        "variables",
                        "变量定义",
                        PageValueKind::Json,
                        "template.variables",
                    ),
                    field(
                        "builtIn",
                        "内置",
                        PageValueKind::Boolean,
                        "template.builtIn",
                    ),
                ],
                vec![list(
                    "templates",
                    "模板",
                    "模板",
                    "prompt_list_templates",
                    PROMPT_COLUMNS,
                    empty(
                        "暂无模板",
                        "读取内置模板或保存一个自定义模板。",
                        Some("list-prompts"),
                    ),
                )],
                empty("暂无模板", "读取模板列表以开始编辑。", Some("list-prompts")),
            )],
            vec![
                action(
                    "list-prompts",
                    "模板列表",
                    "prompt_list_templates",
                    PageActionKind::Query,
                    "读取并确保内置模板存在。",
                    &[],
                ),
                action(
                    "preview-prompt",
                    "预览模板",
                    "prompt_preview_template",
                    PageActionKind::Preview,
                    "替换已提供变量并报告 missingVariables。",
                    &[required_field(
                        "request",
                        "预览请求",
                        PageValueKind::Json,
                        "args.request",
                    )],
                ),
            ],
        ),
    };

    let layout = layout_for(route, &sections, &actions);
    PageModel {
        route,
        summary,
        layout,
        sections,
        actions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeApp;

    #[test]
    fn every_route_has_a_business_model() {
        for route in Route::ALL {
            let model = route.page_model();
            assert_eq!(model.route, route);
            assert!(!model.summary.is_empty());
            assert!(!model.sections.is_empty(), "missing sections for {route:?}");
            assert!(!model.actions.is_empty(), "missing actions for {route:?}");
            assert!(model.field_count() > 0, "missing fields for {route:?}");
            for section in &model.sections {
                assert!(!section.title.is_empty());
                assert!(!section.status.code.is_empty());
                assert!(!section.empty.title.is_empty());
                for list in &section.lists {
                    assert!(!list.columns.is_empty());
                    assert!(!list.empty.title.is_empty());
                }
            }
        }
    }

    #[test]
    fn every_route_has_an_explicit_layout_variant_and_panels() {
        let expected = [
            (Route::Home, "dashboard"),
            (Route::Projects, "project-workspace"),
            (Route::Growth, "growth-workspace"),
            (Route::Timeline, "event-timeline"),
            (Route::Audit, "audit-review"),
            (Route::Graph, "relation-graph"),
            (Route::Search, "search-workbench"),
            (Route::Indexing, "indexing-pipeline"),
            (Route::Backup, "backup-transfer"),
            (Route::Simulation, "simulation-report"),
            (Route::Agent, "agent-console"),
            (Route::Settings, "settings-form"),
            (Route::Help, "command-help"),
            (Route::Diagnostics, "diagnostics-console"),
            (Route::Jobs, "job-queue"),
            (Route::PromptTemplates, "prompt-library"),
        ];
        for (index, (route, variant)) in expected.iter().copied().enumerate() {
            for (other_index, (_, other_variant)) in expected.iter().copied().enumerate() {
                if index != other_index {
                    assert_ne!(variant, other_variant, "duplicate layout variant");
                }
            }
            let model = route.page_model();
            assert_eq!(
                model.layout.variant, variant,
                "layout variant for {route:?}"
            );
            assert_eq!(model.layout.panels.len(), model.sections.len());
            assert_eq!(
                model.layout.action_placements.len(),
                model.actions.len(),
                "action placement count for {route:?}"
            );
            for panel in &model.layout.panels {
                assert_eq!(panel.id, panel.section_id);
                assert!(model
                    .sections
                    .iter()
                    .any(|section| section.id == panel.section_id));
            }
            for placement in &model.layout.action_placements {
                let action = model
                    .action(placement.action_id)
                    .expect("every layout action must be a model action");
                assert_eq!(placement.placement, action.placement);
            }
        }
    }

    #[test]
    fn field_controls_and_list_presentations_are_business_specific() {
        for route in Route::ALL {
            let model = route.page_model();
            for section in &model.sections {
                for field in &section.fields {
                    let expected = match field.kind {
                        PageValueKind::Identifier => FieldControlKind::ReadOnly,
                        PageValueKind::Text => FieldControlKind::TextInput,
                        PageValueKind::MultilineText => FieldControlKind::TextArea,
                        PageValueKind::Number | PageValueKind::Decimal => {
                            FieldControlKind::NumberInput
                        }
                        PageValueKind::Boolean => FieldControlKind::Toggle,
                        PageValueKind::Timestamp => FieldControlKind::DateTime,
                        PageValueKind::Status => FieldControlKind::StatusBadge,
                        PageValueKind::Tags => FieldControlKind::TagInput,
                        PageValueKind::EntityReference => FieldControlKind::EntityPicker,
                        PageValueKind::Json => FieldControlKind::JsonEditor,
                    };
                    assert_eq!(field.control, expected, "control for {}", field.id);
                }
                for list in &section.lists {
                    let expected = match list.id {
                        "recent-entries" | "projects" => Some(ListPresentation::CompactTable),
                        "characters" | "axioms" | "templates" | "providers" => {
                            Some(ListPresentation::Cards)
                        }
                        "events" | "phases" => Some(ListPresentation::Timeline),
                        "facts" | "conflicts" => Some(ListPresentation::ConflictTable),
                        "relations" | "relation-neighborhood" | "neighborhood" => {
                            Some(ListPresentation::Graph)
                        }
                        "results" => Some(ListPresentation::ResultCards),
                        "chunks" | "growth-records" => Some(ListPresentation::Pipeline),
                        "jobs" => Some(ListPresentation::StatusQueue),
                        "archive" => Some(ListPresentation::Transfer),
                        "context" => Some(ListPresentation::Preview),
                        _ => None,
                    };
                    if let Some(expected) = expected {
                        assert_eq!(list.presentation, expected, "presentation for {}", list.id);
                    }
                }
            }
        }
    }

    #[test]
    fn layout_panel_kinds_match_route_intent() {
        let expected = [
            (Route::Home, PanelKind::Summary),
            (Route::Projects, PanelKind::Table),
            (Route::Growth, PanelKind::Inspector),
            (Route::Timeline, PanelKind::Timeline),
            (Route::Audit, PanelKind::Audit),
            (Route::Graph, PanelKind::Graph),
            (Route::Search, PanelKind::Results),
            (Route::Indexing, PanelKind::Pipeline),
            (Route::Backup, PanelKind::Transfer),
            (Route::Simulation, PanelKind::Report),
            (Route::Agent, PanelKind::Console),
            (Route::Settings, PanelKind::Settings),
            (Route::Help, PanelKind::Help),
            (Route::Diagnostics, PanelKind::Diagnostics),
            (Route::Jobs, PanelKind::Queue),
            (Route::PromptTemplates, PanelKind::Form),
        ];
        for (route, kind) in expected {
            let model = route.page_model();
            assert!(
                model.layout.panels.iter().any(|panel| panel.kind == kind),
                "missing {kind:?} panel for {route:?}"
            );
        }
    }

    #[test]
    fn page_actions_match_existing_route_command_contract() {
        for route in Route::ALL {
            let expected = route.actions();
            let actual = route.page_model().actions;
            assert_eq!(actual.len(), expected.len(), "action count for {route:?}");
            for ((id, label, command), action) in expected.iter().zip(actual.iter()) {
                assert_eq!(*id, action.id);
                assert_eq!(*label, action.label);
                assert_eq!(*command, action.command);
                assert!(NativeApp::command_names()
                    .iter()
                    .any(|name| *name == action.command));
            }
        }
    }
}
