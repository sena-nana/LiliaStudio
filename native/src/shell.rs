//! 应用外壳：DesktopShell 组装 — 侧栏导航、标题栏开关、状态条与命令面板。
//! 视图只读 store 并通过 `RuntimeProgramContext` 派发消息，不做任何 I/O。

use nana_ui::runtime::view::{column, dynamic, row, text, widget, IntoView};
use nana_ui::runtime::view::{EntityRef, Store};
use nana_ui::runtime::{
    Activate, CommandPalette, CommandPaletteEvent, CommandPaletteItem, DesktopShell, Icon,
    IconButton, OverlayHost, SidebarFrame, SidebarRow, SidebarRowState, SidebarSection, StatusBar,
};
use nana_ui::{
    ActionId, RegionId, RegionRole, RegionState, RuntimeProgramContext, WorkspaceLayout,
    WorkspaceModel,
};

use crate::state::{
    AppMessage, AppState, AppStateStoreFields, ThemeChoice, NAV_FOOTER, NAV_GROUPS,
};
use crate::{views, Route};

/// 侧栏宽度契约（Tauri appShell.ts SIDEBAR_CONFIG）：默认 240，可拖 180–480。
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 240.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 180.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 480.0;

/// 在默认布局基础上把侧栏（Resources 区）换成 Ameya 的宽度契约；
/// `initial` 为持久化的用户宽度（缺省回默认 240）。
fn sidebar_layout(initial: Option<f32>) -> WorkspaceLayout {
    let regions = WorkspaceLayout::default().regions().to_vec();
    let regions = regions
        .into_iter()
        .map(|region| {
            if *region.id() == RegionId::Resources {
                RegionState::new(RegionId::Resources, RegionRole::Resources)
                    .size(initial.unwrap_or(SIDEBAR_DEFAULT_WIDTH))
                    .min_size(SIDEBAR_MIN_WIDTH)
                    .max_size(SIDEBAR_MAX_WIDTH)
                    .collapsible(true)
                    .resizable(true)
            } else {
                region
            }
        })
        .collect::<Vec<_>>();
    WorkspaceLayout::new(regions).unwrap_or_default()
}

/// 面板条目 = 14 条命令（Tauri 契约）：新建项目 / 搜索 + 全部路由。
/// id 即 `Route::path()`，供 `AppMessage::PaletteCommand(&str)` 直接匹配。
fn palette_commands() -> Vec<(&'static str, &'static str, AppMessage)> {
    let mut commands = vec![
        (
            "new-project",
            "新建项目",
            AppMessage::Navigate(Route::Projects),
        ),
        ("search", "搜索", AppMessage::Navigate(Route::Search)),
    ];
    for route in NAV_GROUPS
        .iter()
        .flat_map(|(_, routes)| routes.iter())
        .chain(NAV_FOOTER.iter())
    {
        commands.push((route.path(), route.title(), AppMessage::Navigate(*route)));
    }
    commands
}

/// 面板选中后派发的消息；未注册的 action 返回 None。
pub fn palette_message(id: &str) -> Option<AppMessage> {
    palette_commands()
        .into_iter()
        .find(|(action, _, _)| *action == id)
        .map(|(_, _, message)| message)
}

/// 全局外壳。`host_ref`/`palette_ref`/`shell_ref` 在视图提交后可解析为实体，
/// app.rs 用它们开关面板、读写工作区（侧栏宽度/双击重置）。
/// `initial_sidebar_width` 来自持久化 kv（ameya.sidebarWidth）。
pub fn shell(
    app: Store<AppState>,
    context: RuntimeProgramContext<AppMessage>,
    host_ref: EntityRef<OverlayHost>,
    palette_ref: EntityRef<CommandPalette>,
    shell_ref: EntityRef<DesktopShell>,
    initial_sidebar_width: Option<f32>,
) -> impl IntoView {
    let model = WorkspaceModel::with_layout(sidebar_layout(initial_sidebar_width));
    widget(DesktopShell::from_model(model).title("Ameya"))
        .entity_ref(shell_ref)
        .title_leading(title_leading(app.clone()))
        .navigation(sidebar_navigation(app.clone()))
        .primary(views::page(app.clone(), context.clone()))
        .bottom(bottom_status(app.clone()))
        .overlay(palette_layer(host_ref, palette_ref))
}

fn title_leading(app: Store<AppState>) -> impl IntoView {
    let sidebar_toggle = widget(IconButton::new(Icon::Sidebar, "切换侧栏"))
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::ToggleSidebar));
    let theme_toggle = dynamic(app.theme(), move |theme| {
        let next = theme.toggle();
        let (icon, label) = match theme {
            ThemeChoice::Dark => (Icon::Appearance, "切换到浅色"),
            ThemeChoice::Light => (Icon::Moon, "切换到暗色"),
        };
        widget(IconButton::new(icon, label))
            .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::SetTheme(next)))
    });
    row().children((sidebar_toggle, theme_toggle))
}

fn sidebar_navigation(app: Store<AppState>) -> impl IntoView {
    let new_project =
        nana_ui::runtime::view::button("新建项目").on_cx(move |_, _: &Activate, cx| {
            cx.dispatch_program_all(AppMessage::Navigate(Route::Projects))
        });
    let search = nana_ui::runtime::view::button("搜索").on_cx(move |_, _: &Activate, cx| {
        cx.dispatch_program_all(AppMessage::Navigate(Route::Search))
    });
    widget(SidebarFrame::new())
        .top(column().children((new_project, search)))
        .body(sidebar_body(app))
}

/// 侧栏主体：分组导航。路由变化时整块重建（低频事件，无谓细粒度）。
fn sidebar_body(app: Store<AppState>) -> impl IntoView {
    dynamic(app.route(), move |active| {
        let groups = NAV_GROUPS
            .iter()
            .map(|(label, routes)| {
                widget(SidebarSection::new(*label)).children(
                    routes
                        .iter()
                        .map(|route| nav_row(*route, *active))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let footer = NAV_FOOTER
            .iter()
            .map(|route| nav_row(*route, *active))
            .collect::<Vec<_>>();
        column().children((groups, footer))
    })
}

fn nav_row(route: Route, active: Route) -> impl IntoView {
    let state = if route == active {
        SidebarRowState::Active
    } else {
        SidebarRowState::Idle
    };
    widget(SidebarRow::new(route.title()).state(state))
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::Navigate(route)))
}

fn bottom_status(app: Store<AppState>) -> impl IntoView {
    let status_text = dynamic(app.status(), |status| text(status.text.clone()));
    let busy_text = dynamic(app.busy(), |ops| {
        text(ops.first().map(|op| op.label.clone()).unwrap_or_default())
    });
    widget(StatusBar::new().chrome(false)).children(row().children((status_text, busy_text)))
}

/// 命令面板常驻 overlay 层；Ctrl+K / 搜索按钮经 activate_overlay 激活。
fn palette_layer(
    host_ref: EntityRef<OverlayHost>,
    palette_ref: EntityRef<CommandPalette>,
) -> impl IntoView {
    let items = palette_commands()
        .into_iter()
        .map(|(id, label, _)| CommandPaletteItem::new(ActionId::new(id), label))
        .collect::<Vec<_>>();
    let palette = widget(CommandPalette::new("命令面板", items))
        .entity_ref(palette_ref)
        .on_cx(move |_, event: &CommandPaletteEvent, cx| {
            if let CommandPaletteEvent::Select(action) = event {
                let id = action.as_str().to_owned();
                if let Some(message) = palette_message(&id) {
                    cx.dispatch_program_all(message);
                }
            }
        });
    widget(OverlayHost::new())
        .entity_ref(host_ref)
        .children(palette)
}
