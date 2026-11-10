# NanaUI API + Spec Reference (rev 596930d)

> **权威参考** — Ameya UI 重写必须遵循的 NanaUI API 与规范。基于本地 checkout
> `/c/Users/wangjunxue/.cargo/git/checkouts/nanaui-9f21799893cf41ff/596930d/`
> (rev `596930d22261bf048f12a94bdcb4f1ee2703e250`，与 `native/Cargo.toml` git 依赖 pin 的 EXACT rev)。
>
> **验证方法**: 每个 API 名字都对照了 crate 源码 (`crates/`) 或 checkout 自带文档 (`docs/`)。
> 标注 `// src: <path>:<line>` 的项直接核对了源码行; 标注 `(doc)` 的项来自 checkout 官方文档
> `docs/components/*.md` / `docs/guide/` / `docs/reference/`（该文档即为此 rev 编写）。
> 标注 **[未验证]** 的项明确未核对到源码 —— 使用前必须自行确认。
>
> 生成者: teammate "nana-spec"。只读审计，未改动 NanaUI checkout 与工作区任何现有文件。

---

## 1. IMPORT MAP

### 1.1 `nana_ui::framework` 不存在 —— canonical 根是 `nana_ui::runtime`

- `crates/nana-ui/src/lib.rs` **没有** `framework` 模块。`crates/nana-ui-runtime/src/lib.rs` 有
  `mod framework;` 但是 **private**（AppContext 等经它 re-export，但路径不可达）。
- 我们 crate 里的 `nana_ui::framework::runtime::…` 能编译，是因为 `native/src/lib.rs:23`
  自己写了 `pub use nana_ui as framework;` —— **framework 是我们自己的别名，不是 NanaUI 的模块**。
- `crates/nana-ui/src/runtime.rs` 模块头写明 **"Canonical application API. Prefer this module over
  crate-root widget re-exports."** 它由两部分组成：
  - `pub use nana_ui_runtime::*;`（全部 runtime 组件 + view 模块）
  - 子模块 `host`（RuntimeDocument 等 scene 类型）、`rich`（富文本词汇）、`perf`、`internal`（doc-hidden）。

**重写时的导入约定**: 组件/视图/状态一律 `use nana_ui::runtime::…` 与 `use nana_ui::runtime::view::…`；
宿主入口（run_runtime、ApplicationState 等）用 crate 根 `use nana_ui::…`；WindowEvent/WindowId 只有
`nana_ui_platform` 一个来源。删除我们自己的 `framework` 别名以免误导。

### 1.2 运行时项（任务清单逐项核对）

| 项 | 验证过的导入路径 | 源码 |
| --- | --- | --- |
| `RuntimeDocument` | `nana_ui::runtime::RuntimeDocument`（= `runtime::host::RuntimeDocument`） | crates/nana-ui/src/runtime.rs:20,58 (`pub use nana_ui_scene::…`) |
| `DocumentId` | `nana_ui::runtime::DocumentId` 或 crate 根 `nana_ui::DocumentId` | 定义 crates/nana-ui-runtime/src/world.rs:322；nana-ui/src/lib.rs:326 |
| `ApplicationState` | `nana_ui::ApplicationState`（feature `hosted`） | crates/nana-ui/src/application.rs:77 |
| `RuntimeApplication` | `nana_ui::RuntimeApplication`（hosted；`RuntimeApplication<State>` 把 ApplicationState 适配成 RuntimeProgram） | crates/nana-ui/src/application.rs:254 |
| `run_runtime` | `nana_ui::run_runtime`（hosted） | crates/nana-ui/src/runtime_host.rs:1042 |
| `WindowDescriptor` | `nana_ui::WindowDescriptor`（hosted；本体在 nana-ui-platform） | crates/nana-ui-platform/src/window.rs:482 |
| `ThemeAppearance` / `CompiledTheme` | `nana_ui::{ThemeAppearance, CompiledTheme}`；`builtin_theme_arc(appearance)` 也在 crate 根 | crates/nana-ui/src/theme.rs（re-export nana-ui-core） |
| `WindowEvent` / `WindowId` | **只能** `use nana_ui_platform::{WindowEvent, WindowId};` —— nana-ui crate 根不 re-export（application.rs:8 自己也是这么导的） | crates/nana-ui/src/application.rs:8 |
| `RuntimeProgramUpdate` | `nana_ui::RuntimeProgramUpdate`（hosted）；构造子 `::default()` `::redraw(id)` `::redraw_all()` `::exit()` `::merge(other)` | crates/nana-ui/src/runtime_host.rs:385-411 |
| `RuntimeProgram` / `RuntimeProgramContext` | `nana_ui::{RuntimeProgram, RuntimeProgramContext}` | runtime_host.rs:555 / application.md |
| `RuntimeRedraw` | `nana_ui::RuntimeRedraw`（`None`/`Window(id)`/`All` + `merge`） | runtime_host.rs（HeadlessReport 字段类型） |
| `NanaApplication` / builder | `nana_ui::{NanaApplication, ApplicationIdentity, DiagnosticsConfig}` | crates/nana-ui/src/application_builder.rs:24-27 |
| `headless` 模块 | `nana_ui::headless`（hosted）——无窗口宿主，见 §7 | crates/nana-ui/src/lib.rs:133 |

### 1.3 视图构造函数（`nana_ui::runtime::view::*`，controls.rs / marks.rs / settings.rs 等）

我们当前已在用的 `nana_ui::runtime::view::{button, column, widget}` 全部正确。完整清单（源码核对）：

**布局/文本/基础** (`crates/nana-ui-runtime/src/view/controls.rs`):
`column() -> El<Stack>` (:356)，`row() -> El<Stack>` (:362)，`text(value) -> El<Text>` (:393)，
`button(label)`，`divider()`，`tabs() -> El<Tabs>`，`tree_view() -> El<TreeView>`，
`list_item(label)`，`progress(max: f64)`，`spinner()`，`avatar(size: f32)`，
`icon_button(Icon, label)`，`chip(label)`，`status_badge(label)`，`empty_state(title)`，
`labeled_value(label)`，`thumbnail()`，`texture()`，`slider(min,max,step)`，`range_span`，`curve(step)`。

**输入控件** (同文件): `text_input()`，`text_area()`，`number_input()`，`checkbox(label)`，
`switch(label)`，`select() -> El<Select>`，`size_field()`，`color_field()`，
`action_menu(label)`，`action_menu_item(label)`，以及 `fields` 模块（typed 字段函数）。

**没有 `card()` / `table()` / `scroll_view()` / `stack()` 视图函数** —— 这些组件用
`widget(Card::new())` / `widget(Table::new())` / `widget(ScrollView::new(ScrollAxes::Vertical))` 形式
（示例: examples/component-gallery/src/bin/ui_benchmark/runner.rs:487）。Stack 的直接构造:
`Stack::column(gap)` / `Stack::row(gap)` / `Stack::fill_column(gap)` / `Stack::fill_row(gap)`
(crates/nana-ui-runtime/src/view_components.rs:6441-6480)。

**标记类** (`view/marks.rs`): `settings_group(title, rows)`，`group_row`，`group_row_on_line`，
`group_block`，`content_line`，`heading_row`，`callout`，`kbd`，`tag`，`footer_bar`，
`scroll_foot_fade`，`CONTENT_INSET`。`view/settings.rs`: `settings_row(label)`。

**节点/组合** (`view/node.rs`): `widget(component) -> El<C>` (:834)，`El::children(children)` (:1006)，
`node_ref()`，`entity_ref()`，`keyed`，`with_refs`，`detached`，`AnyView`，`IntoView`，`Children`，
`NodeRef`，`EntityRef`，`Keyed`，`Mount`。

**响应式** (`view/reactive.rs`): `signal(v) -> Signal<T>` (:732)，`computed(f) -> Computed<T>`
(:746, T: PartialEq)，`constant(v)` (:1259)，`watch_effect(f) -> Effect` (:791)，`on_cleanup` (:798)，
`on_mount(f: FnOnce(&mut AppContext))` (:859)，`provide<T>(value)` (:811)，
`use_context<T: Clone>() -> Option<T>` (:834)，`untrack` (:884)，`Readable` trait (:1311)。

**store** (`view/store.rs`): `store(value) -> Store<T>` (:505)，
`store_with_history(value, limit)` (:526)，`Store`，`StorePath` (:282)，`StoreList<T>`（blanket impl
:954），`KeyedList`（`.at(&key)` :819，`.items()` :830，`.each(fn)` :850，`.each_virtual` :865），
`Item`（`.id()` :747），`Subfield`。宏 `#[derive(Store)]` 在 nana-ui-view-macros
(crates/nana-ui-view-macros/src/lib.rs:24)，生成 `<Name>StoreFields` trait —— **使用处要 import 该 trait**。

**结构** (`view/structural.rs`): `each`，`when`（`.otherwise()` 需 `WhenExt`），`dynamic`；
`Each::gap/horizontal/transition/key/container`，`When::otherwise/key/transition/keep_alive/grow`，
`Dynamic::keep_alive/max/transition/key/grow`。虚拟长列表 `view/each_virtual.rs`:
`each_virtual`，`virtual_list_ref`，`EachVirtual::scroll_view(scroll)` (:190)。

**其他**: `view/resource.rs`（`resource`，`suspense`），`view/selection.rs`（`segmented`，
`segmented_option`），`view/teleport.rs`（`teleport`），`view/transition.rs`（`Implicit`，`Presence`，
`Transition`）。宏（feature `view-macro`）: `view!`，`css!`，`stylesheet!`，`text!`（格式化插值，
信号变化时重求值），`#[component]`。

**`flush_reactive`**: 不是自由函数 —— `AppContext::flush_reactive(&mut self)` 方法
(crates/nana-ui-runtime/src/framework/reactive.rs:359)。按轮执行: 先 `watch_effect`，再 `each`/`when`
结构更新，最后**一次** commit 批量提交所有节点变更。输入路由在每个事件末尾调用、每帧开头
`take_system_work` 再调 —— **on_mount 里不用手动刷**（docs/guide/essentials/lifecycle.md:93）。

### 1.4 El 通用字段/事件

`El<C>` 上（源码+docs）: `.gap(px)`，`.visible(bool|信号)`，`.on_activate(\|e: &Activate\|)`，
`.on(\|e: &Event\|)`（typed 事件），`.on_cx(\|component, &Event, ViewContext\|)`，
`.model(writable)`（双向绑定），`.node_ref(NodeRef)`，颜色/圆角角色 `.foreground/.background/
.border/.radius(SemanticColorRole/RadiusTier)`，`.key(k)`，`.css(...)`/`.style(NodeStyle)`。

---

## 2. SHELL COMPONENTS

格式: 构造函数 + builder + 事件 + 最小片段。除注明外均核对 `docs/components/<name>.md`（此 rev
官方文档）+ `crates/nana-ui-runtime/src/lib.rs` 导出（导出名 100% 源码核对）。

### DesktopShell（外壳主入口）
`crates/nana-ui-runtime/src/shell.rs` 导出。构造: `DesktopShell::new()` / `DesktopShell::from_model(&model)`。
Builder: `.title(text)`（自动构建 AppTitleBar），`.title_center_width(...)`，
`.title_window_controls(false)`，`.workspace_corners(bool, 默认 true)`。模型:
`WorkspaceModel` + `WorkspaceMutation`（`.apply(mutation, now)`），WorkspaceController 把指针/时钟
转成 mutation。**不要改框架建的 Workspace 节点**。slot 装配器 `assemble_desktop_shell`。
事件: 无自有事件（子组件事件冒泡按各组件）。

```rust
// slots 在 El<DesktopShell> 上（crates/nana-ui-runtime/src/view/shell.rs，源码核对）
fn shell(page: impl IntoView + 'static, nav: impl IntoView + 'static) -> impl IntoView {
    widget(DesktopShell::new().title("Ameya"))
        .navigation(nav)      // 还有 title_leading/title_center/title_trailing/
        .primary(page)        //     navigation_footer/inspector/bottom/overlay/status/
}                             //     region(RegionId, view)
```

### AppShell（无标题栏外壳）
`AppShell::new()`；slots `.title_bar/.body/.overlay`（overlay 绝对定位，仅可见子项收指针）。
对话框走 `activate_overlay` 或子组件 `.open/.model`。`assemble_app_shell`。

### AppTitleBar（标题栏）
`AppTitleBar::new(title)`；`.drag_enabled(bool)`，`.show_window_controls(bool)`，
`.native_controls(bool)`，`.maximized(bool)`，`.transparent(true)`，`.over_media(true)`，
`.center_width(...)`，`.controls_capsule(true)`，`.edge_inset(SpacingStep::Sm)`；slots
`.leading/.center/.trailing`。常量 `TITLE_BAR_HEIGHT`。`assemble_app_title_bar`。

### Workspace（多区域工作台）
`Workspace::new()` = `from_model(&WorkspaceModel::new(), [])`；`.slot(RegionId, content)`。
RegionId 内置: `GlobalNavigation, SectionNavigation, Resources, PrimaryToolbar, Primary,
Inspector, Diagnostics` + `Custom(String)`。`model.apply(mutation, now)`、`refresh_from_model`；
分隔条 8px 命中区，260ms 折叠过渡；`WorkspaceGeometry` 给逻辑/物理矩形；重复 region id 注册被拒。

### Sidebar 三件套（侧栏）
- `SidebarFrame::new()`；slots `.tools/.top/.body/.footer`；body 自动包
  `SidebarFrame::vertical_body_scroll()`（纵向 ScrollView）；有 `gap` 字段。
- `SidebarSection::new("资源")`；`.count(n)`，`.collapsible(true)`，`.empty_text("…")`，`.tools` slot；
  `SidebarSectionState`。
- `SidebarRow::new(label)`；`.state(SidebarRowState::{Idle,Active,AncestorActive,Disabled})`，
  `.depth(n)`，`.disclosure(expanded)`，`.tone(SidebarRowTone::{Default,Warning,Error})`，
  `.tools(view)`，`.tools_rest(RowToolsRest::Reserve)`，`.keep_tools(true)`。激活走
  `Activate`/`activate_sidebar_row` action —— **路由归属应用**。
  辅助: `sidebar_row_depth_inset`，`sidebar_section_tool_button`，`sidebar_top_bar_tool_button`，
  `SidebarFooter`，`SidebarSectionSlots`。

```rust
fn sidebar() -> impl IntoView {
    widget(SidebarFrame::new())
        .body(widget(SidebarSection::new("库").count(2).collapsible(true))
            .children(column().children((
                widget(SidebarRow::new("工程 A").state(SidebarRowState::Active)),
                widget(SidebarRow::new("角色设定").depth(1)),
            ))))
}
```

### SplitPane / PaneSection / PaneChrome
- `SplitPane::from_model(model, first, second)`（`SplitPane::new(&model)` 也有）；
  `SplitPaneModel::new(SplitAxis::Horizontal|Vertical, default_size, min, max)`；
  `.surface(SemanticColorRole)`，`.handle(node)`，`.apply(SplitPaneMutation)`；slots `.first/.second`；
  `assemble_split_pane`；模型可读回持久化。
- `PaneSection` slots `.header/.tabs/.body`；`PaneChrome` slots `.header/.tabs/.action(PaneChromeAction,
  view)/.body`（`PaneChromeActionKind`）。

### StatusBar / Toolbar
`StatusBar::new()`；`.label(a11y 名)`，`.chrome(false)`；children = 读数项；role Status live region。
注意与 DesktopShell 的 `.status` slot（toast 宿主位）不是一回事。`Toolbar` + `ToolGroup(Mode)` 也在
bars.rs。

### CommandPalette
`CommandPalette::new(title, items)`；`CommandPaletteItem::new(ActionId, label).shortcut("⌘S")
.category("…")`；`.placeholder/.empty_label/.filtered_items(true)/.query(...)/set_query`。
`navigate(ActionPickerNavigation::{Previous,Next,First,Last,Confirm,Dismiss})`；Confirm →
`CommandPaletteEvent::Select(action)`；输入 → `Search`。经 `activate_overlay(host, palette)` 打开、
`dismiss_overlay(host)` 关闭；role dialog。

### Dialog / ConfirmDialog / Drawer / ModalSurface
- `Dialog::new(title)`；`.description(...)`，
  `.size(DialogSize::{Compact 420, Default 520, Medium 600, Wide 680, Workspace 1080, Width(len),
  capped(px,vw)})`，`.close_policy(DialogClosePolicy::requests_only())`，`.danger(true)`，
  `.title_icon(view)`，`.title_content(view)`，`.initial_focus_on(entity_ref)`；slots
  `.body/.footer/.close_action`。声明式开关 `.open(bool)` / `.model(Signal<bool>)`（`ModalOpen`）。
  事件: `DialogCloseRequested(trigger)`，`DialogToggled(open)`。`DialogRecipe` 在 ThemeDefinition。
- `ConfirmDialog::new(title, message)`；`.confirm_label/.cancel_label/.danger(true)/.size/
  .close_policy/.initial_focus(ModalInitialFocus)/busy`；slots `.secondary/.body/.footer/…`。
  `AppContext::assemble_confirm_dialog(…)` 在 slot 空时自动建按钮；`set_confirm_slots` 预填；
  事件 `ConfirmIntent`。
- `Drawer::new(title)`；`.side(DrawerSide::{Left,Right,Bottom})`（默认 Right），`.description`，
  `.close_policy`，`.initial_focus`；slots `.title_icon/.title_content/.body/.footer/.close_action`；
  `assemble_modal_slots`；开法同 Dialog。共享: `ModalSurface`，`ModalBehavior`，`ModalInitialFocus`。

### ContextMenu / ActionMenu / AnchoredActionMenu
- `ContextMenu::new(anchor_x, anchor_y)`（逻辑像素，默认 open）；`.searchable(true)`，
  `.items([ContextMenuItem::new(value, label).hint(...).icon(Icon).image(slot).disabled(true)
  .danger(true)])`；value 用 `/` 分隔 = 一层树；`.query/set_query/query_text`；
  `place_in(room)` 自动翻转；挂 OverlayHost 下，`activate_overlay`。事件: `ContextMenu(Event)`。
- `ActionMenu::new()`；`.trigger(text)`，`.trigger_icon(icon,label)`，`.bare_trigger(true)`，
  `.trigger_style(NodeStyle)`；children = `ActionMenuItem`（视图函数 `action_menu_item(label)`，
  `.on_activate(...)`，`.danger`，`.disabled`，`.accessible_name`，`.leading_image`）；open 状态控件
  自管。`AnchoredActionMenu` + `resolve_anchored_origin`（menus.rs）。
- 常量 `MENU_OVERLAY_Z_INDEX`。

### Popover
`Popover::new()`；`.trigger(text)` / `.trigger_icon(icon,label)` / `.bare_trigger(true)` /
`.trigger_style(style)` / `.placement(PopoverPlacement::{Top,Bottom(默认),Left,Right})` /
`.alignment(PopoverAlignment)` / `.width(...)` / `.padding(...)`；children = 面板内容；溢出自动翻
边；open 自管（不走 activate_overlay）。事件: `PopoverClosed`，`PopoverToggled`。

### Dropdown / Select / SearchDropdown（三兄弟，别混）
- `Dropdown::single(value)` / `Dropdown::multiple(values)`（**`Dropdown::new` 是 private**）；
  `DropdownOption::new(value,label).hint(...).disabled(true)`；字段 placeholder/size/disabled/
  loading/invalid/opened/highlighted/menu_scroll/bare_trigger；事件 `DropdownEvent::{Select(单选),
  Toggle(多选), Opened, Closed}`。
- `Select`（视图函数 `select()`）：`.placeholder(...)`，`.options(Vec<SelectOption> 或信号)`，
  `.model(Signal<Option<Arc<str>>>)`（v-model），`.label(...)`，`.labelled_by(node_ref)`，
  `loading/disabled`；`SelectOption::new(value,label).disabled(true).hint(text).image(slot)`；
  **控件自写选中值**。事件: `SelectChanged`，`SelectToggled`。
- `SearchDropdown::new(Some("main"))`；`.placeholder/.options([SearchDropdownOption::new(value,
  label).hint(...)])/.query(...)`；`query_text()`/`visible_indices`；label+value+hint 不区分大小写
  子串匹配。事件: `SearchDropdownEvent::{Select, Closed}`。

### Toast / ToastStack
- `Toast::new(title, ToastTone::{Info,Success,Warning,Danger})`；`.description(...)`，
  `.dismissible(true)`，`.timeout(Duration)`，`.progress(ToastProgress::{Determinate(0.0..=1.0),
  Indeterminate})`，`.action(label)`，`.icon(Icon)`，`.close_label(...)`；
  `Toast::place_in(viewport, align, max_width, insets)`。事件: `ToastDismissed`，`ToastAction`。
  `AppContext::assemble_toast(toast)` → 组合形态（常驻，第二次幂等为 false）。
- `ToastStack::new()`；`.max_visible(3)`，`.label("通知")`；`ToastStack::place_in(viewport,
  PopoverAlignment::End, max_width, insets, max_height)`；**不要作为 activated overlay 挂在
  OverlayHost 下 —— 普通定位节点**。`AppContext::push_toast(stack, toast) -> entity`，
  `remove_toast`，`clear_toasts`，`pulse_toast`，`shown_toasts(stack)`；measure-then-show 排队；
  modal z=1000 交互。`ToastRecipe`/`ToastMotion` 见 docs/reference/theme.md。

```rust
// 打 toast（doc 模式）
let stack = cx.mount_view_root(id, || widget(ToastStack::new().max_visible(3))).unwrap();
let toast = Toast::new("已保存", ToastTone::Success).description("工程 A");
let entity = cx.push_toast(stack, toast);
cx.on(entity, |_, _: &ToastDismissed, _| { /* app 清理 */ });
```

### Panel（非模态面板；无 docs 页，源码核对 crates/nana-ui-runtime/src/panel.rs）
`Panel::new(label)`；`.kind(CardKind)`(:91)，`.painter(...)`(:97)，`.close_on_escape(bool, 默认 true)`
(:102)，`.focus_on_open(bool)`(:108)，`.style(NodeStyle, 默认 fill_column z_index 100)`(:86)，
`.viewport(...)`(:116)，`.bounds(...)`(:139)。children = 标题/动作/体（通常 ScrollView）。直接挂
OverlayHost，`activate_overlay/dismiss_overlay`；关闭/退场用 `OverlayClosing`/`OverlayChanged`。
业务导航与钉住归应用。`PanelEdge::{Left,Right}` 与 `PanelInsets{top,right,bottom,left}` 是保留
chrome 的共享合同（Toast::place_in / ToastStack::place_in 同用）。

---

## 3. DATA DISPLAY

### Table（可排序）+ VirtualTableLayout
- `Table::new().label("文件")`（role=table）；`TableRow::new().selected(bool)`；
  `TableCell::new(value).column_header(true)`（否则数据格；默认可点+可聚焦）。rows/cells 是子节点，
  Table 本身无指针。`TableCellFocused` 事件。
- 排序/列宽归 **VirtualTableLayout**（`nana_ui::runtime` 根 re-export，virtual_list.rs）:
  `VirtualTableLayout::new(row_extents, [TableColumn::new(key, extent).sortable(true)
  .limits(min,max).resizable(true)])`；`layout.toggle_sort(column) -> TableSort`（asc → desc → 第三次
  清空；非 sortable 拒绝）；`set_sort(None|Some)`；`move_column(from,to)`。**app 自己重排数据并重排
  子节点**，layout 只发排序状态（TableSort 从 layout 发布）。

```rust
fn files_table(files: Signal<Vec<FileRow>>, sort: &mut VirtualTableLayout) -> impl IntoView {
    widget(Table::new().label("文件")).children(/* header row + data rows */)
}
```

### TreeView
`TreeView::new(nodes)`（nodes: `impl Into<TreeNode<Arc<str>>>`；也有 `tree_view()` 视图函数）；
`TreeNode::leaf(id,label)` / `TreeNode::branch(id,label,expanded,children)`。node 级:
`.icon .selected .disabled .hide_toggle(col) .hide_toggles()`；tree 级: `.label`，`.size`，
`.multiple(true)`（Ctrl/Cmd 单切、Shift 范围），`.columns([TreeToggleColumn::new(icon,name)])`，
`.reorderable(true)`，`.renamable(true)`，`.style`。事件: `TreeViewEvent::{Toggle, Select}`（用
`apply_event` 应用），`TreeSelectionChanged{selected,active}`，`TreeToggleChanged{id,column,on}`，
`TreeDropRequested{rows,intent:Before|After|Inside}`，`TreeDragReleased{rows,x,y}`，
`TreeRenameRequested{id,text}`（配合 `begin_tree_rename/end_tree_rename`）。读回:
`visible_rows()`, `selected_id/selected_ids/active_id`, `renaming`。导航:
`navigate(TreeNavigation)`。a11y Tree + TreeItem 虚拟节点。

### List / ListItem
`List::new().label("最近文件").style(NodeStyle)`（role=list，children=ListItem；列表本身无指针/焦点，
滚动外包 ScrollView）。`list_item(label)`：`.detail("行尾")`，`.note("紧跟主文本")`，`.selected`，
`.disabled`，`.read_only(true)`，`.tooltip(IconButtonTooltip)`，
`.role(ListItemRole::{ListItem,Button,ToggleButton})`，`.has_popup(true)`，`.keep_tools`，
`.tools(icon_button(..))`，`.tools_rest(RowToolsRest::{Yield,Reserve})`，`.on_activate(fn)` /
`.on_cx(&Activate, ViewContext)`。唯一事件 `Activate`。

### ReorderList（feature `controls`）
`ReorderList::new([ReorderItem::new("a","第一项").selected(true)]).label("顺序")`；字段 `.spacing`，
`.size`，`.tree_drop(true)`，`.reorder(false)`（仅选择 → Select 事件），`.transfer_group("…")`（跨
表拖拽，同文档内）。`ReorderItem::new(value,label).draggable(false)/.drop_target(true)/.selected/
.disabled/.tools(id)/.movable(false)`。事件 `ReorderListEvent::{Reorder{source,before}, Select,
Cancelled, TreeDrop{source,intent}}`。**app 重排并写回 items**。

### Card
`Card::new()`；`.title("工程")`（=label=a11y 名），`.kind(CardKind::{Surface,Raised,Outlined,Flat,
Selected})`，`.padding(px)`（清 panel inset），`.height(...)`，`.loading(true)`（a11y busy），
`.painter(custom)`（内部 `cx.draw_default()`）。非交互表面；children = 内容；默认 RadiusTier::Md。
`InteractiveCard`（form_surfaces.rs）用于可点卡。

### EmptyState / StatusBadge / Chip / Progress / Skeleton / Spinner / Avatar / Tooltip / ValidationMessage
- `empty_state(title)`；`.message("…")`，`.icon(Icon::Folder)`，`.compact(true)`，`.action(button…)`
  slot。字段: title/message/icon/compact；无事件。
- `status_badge(label)`；`.tone(StatusTone::{Neutral,Info,Success,Warning,Danger})`，`.compact(bool,
  构造默认 true；false 字更大)`。
- `chip(label)`；`.selected(bool)`，`.on_activate`（**selected 归 app**），`.disabled`；组件级:
  `dismissible(true)`（bind），`close_label`，`Chip::size`（默认 ControlSize::Small）；事件
  `ChipDismissed`（移除归 app）。
- `progress(max: f64)`；`.value(信号/常量/闭包, clamp 0..max)`，`.label(...)`，`.indeterminate(true)`；
  事件 `ProgressCancelled`（ProgressMode）。
- `Skeleton::new(width: LengthSpec, height: f32)` / `Skeleton::fill_width(height)`；`.width/.height`；
  脉动 1↔0.48。
- `spinner()`；`.label(Arc<str>)`（同时是 a11y 名）；组件级 `Spinner::size`。
- `avatar(size: f32)`；`.resource(TextureSlotName)`，`.label(...)`；组件级 `pointer_events(true)`
  才有命中。
- `Tooltip::new(label)` / `Tooltip::with_config(label, TooltipConfig{placement: FollowCursor|Top|
  Right|Bottom|Left, delay 350ms, max_width 280, gap, viewport_padding})`；IconButton::with_tooltip
  用默认配置。
- `ValidationMessage::new(message, ValidationIntent::{Warning,Danger})`；`.compact(false)`，
  `.intent(...)`，`.style(...)`；`AppContext::validity_of(root)` 按文档序返回 invalid+enabled 字段
  （`FormValidity`）。

### NativeMarkdown（feature `rich-text`）
`NativeMarkdown::new()` / `from_source(md)` / `parse(md)` / `from_blocks(MarkdownBlock)`；
`blocks()` 读回。GFM+表格+删除线+任务列表+数学；mermaid/math fence 原生渲染，presenter slot
`MERMAID_PRESENTER="mermaid"` / `MATH_PRESENTER="math"`；`assemble_markdown` slot 装配器。

### CalendarHeatmap（feature `calendar`）
`CalendarHeatmap::new([CalendarHeatmapDatum::new("2026-09-01", 1.0).data(payload)])
.week_starts_on(1).label("提交")`；日期/值/层级归 app。

### Chart（feature `charts`）
`Chart::new(option: ChartOption).label("…")`；词汇在 `nana_ui::runtime::chart::{ChartOption, Axis,
Legend, BarSeries, LineSeries, PieSeries, ScatterSeries, RadarSeries, GaugeSeries, SeriesData}`
（= nana-ui-charts）。
```rust
ChartOption::new()
    .x_axis(Axis::category(["一","二","三"]))
    .y_axis(Axis::value())
    .series(BarSeries::new("字数", [120.0, 340.0, 90.0]))
    .series(LineSeries::new("目标", [200.0, 200.0, 200.0]).smooth(true))
```
宽填充、高默认 300；NaN=断点，`.connect_nulls(true)`。事件 `ChartEvent`，视觉 `ChartHoverVisual` /
`ChartTooltip`。

### GraphCanvas / GraphMinimap（feature `graph-canvas`）
`GraphCanvas::new(canvas_id, GraphModel)`；`GraphModel::empty()` / `GraphModel::new(nodes,edges) ->
Result<_, GraphModelError>`。受控画布（viewport+selection+指针在控件，model 归 app）；事件
`GraphCanvasEvent`；`set_viewport`；模型变更要 `set_model`。自定义渲染器 key
`GRAPH_CANVAS_RENDERER`（宿主必须注册）。`assemble_graph_canvas_contents`。
`GraphMinimap::new(model).canvas_size(GraphSize::new(640.,480.)).label("…")`；`ViewportRequested`
→ app 写回 `GraphCanvas::set_viewport` + minimap。

### TimelineEditor
`TimelineEditor::new([TimelineTrack::new(id,label,[TimelineKey::new(time,value)]).expanded(true)
.depth(1)]).fps(30.0).playhead(0.5)`；`.snap(默认 true)`；时间轴单位 `TimelineUnit::Frames/Seconds`。
事件: `TimelinePlayheadChanged{time,committed}`，`TimelineKeysMoved{keys,dt,dv,committed}`，
`TimelineSelectionChanged{keys}`，`TimelineTrackToggled`，`TimelineClip*`（`TimelineClip::new(id,
start,length,label)`）。`cx.fit_timeline(entity)`；选中可读。

### PropertyGrid
`PropertyGrid::new([PropertySection::new(id,"标题",[PropertyRow::new(id,label).mixed(true)
.disabled(Some("原因"))])]).label_width(96.0)`；行控件 = children 按 row id keyed（`widget(
ScrubField::new(12.0).mixed(true)).key("x")`）；`ScrubField::new(v).unit("°").range(a,b)`。事件:
`PropertySectionToggled{id,collapsed}`，`PropertyLabelWidthChanged{width}`；读回
`collapsed_sections()`。删除行的控件是隐藏不是销毁；重渲染保留用户折叠/列宽。

### DynamicForm
`DynamicForm::new([DynamicFormField::section(id,title), .field(id,label,value)
.binding_identity("x").hint(..), .toggle(id,label,bool), .choice(id,label,value,
[DynamicFormOption::new(v,l)]), .action(id,label).danger(true)])`；`.on(\|&DynamicFormEvent\| …)`。
**稳定 id key（非空、不含 '/'）保控件/焦点/undo**；逐字段校验结果。

### SettingsGroup / SettingsRow（settings.rs + marks.rs）
```rust
settings_group(Some("输入".into()), vec![
    group_row_on_line(settings_row("发送").control(text_area())),
    group_row_on_line(settings_row("模型").control(select())),
]).inset().footer(footer_bar(/*…*/))
```
`settings_row(label)`（内部 `SettingsRow::new("").stack_below(280.0)`）：`.control(view)`，`.hint(...)`，
`.stacked(true)`，`.stack_below(width)`，`.divided(true)`，`.first_in_group/.last_in_group`（SettingsGroup
管），`.loose`。`SettingsGroup` 管可见成员间发丝线；`group_row_on_line` = content_line padding
（CONTENT_INSET = SpacingStep::Xxxl）；`group_block(children).quiet().gap(step)`；用 group_row 上
`.visible(cond)` 隐藏行。整页装配: `SettingsPage(Assembly)` / `SettingsSidebar(Assembly)` /
`AppearanceSection(Assembly)` / `AboutSection` / `apply_appearance_event` / `SettingsCard`。

### FormField
`FormField::new(label).hint("…").error("…")` + `.control(view)` slot（另有 leading/trailing/
secondary slots）；label = a11y 名；error 设了 → a11y invalid；列布局宽 100%。

---

## 4. RICH TEXT

### 模型（app 拥有文档值）
- `RichText`（crates/nana-ui-core/src/rich_text.rs:870）: `{text: Arc<str>, paragraph_alignments,
  spans(按字节区间的 RichSpanStyle), objects(在 OBJECT_REPLACEMENT 字符处), rubies}`。区间是**字节
  offset**，自动对齐字符边界。
- 构造: `RichText::new("纯文本")`；`RichText::builder().plain("今天也").push("辛苦了",
  RichSpanStyle::new().bold().color(PaintColor::srgb([1.0,0.4,0.5,1.0]))).build()`；
  `rich.with_span(0..9, style)`。
- `RichObject::image(id,source,w,h)` / `texture(id,slot,w,h)` / `chip(id,label,kind)`（零宽标记，
  仅编辑器） / `tag(id,label,kind)` / `label(id,label,RichLabelStyle{fill,text,scale,icon,..})`
  （app 染色的 mention 药丸，编辑器+视图同宽）。
- `RichSpanStyle` builder: `family`，`size(px)/size_em`，`family_alias`，`weight(u16)`，
  `stretch(50-200%)`，`bold`，`italic`，`letter_spacing`，`features`，`color(PaintColor)`，
  `background_color`，`underline`，`line_through`，`decoration_color`，`emphasis(TextEmphasisStyle)/
  emphasis_position/emphasis_color`，`stroke(RichTextStroke::new(width,color).join/placement)`，
  `shadow(RichTextShadow::new([x,y],blur,color).spread)`，`effect(u16)`。变更代价分层:
  shape(family/size/weight/stretch/italic/letter_spacing/features → 重新排版) / emphasis(→仅重布局) /
  paint(color/background/decoration → 仅重绘) / effect(合成器)。
- 导入: `nana_ui::runtime::rich::{RichText, RichTextBuilder, RichSpanStyle, PaintColor, RichObject,
  RichTextStroke, RichTextShadow}`（另有 AttributedRanges, OBJECT_REPLACEMENT 等）。

### RichTextEditor（编辑器；值归 app）
`RichTextEditor::new(RichText::new("说点什么")).font_size(20.0)`。**每次变更发
`RichTextEditorEvent::Changed(RichText)`；app 存值并在重建时交回；相等值不提交。** 编辑器只持编辑
态（选区、IME preedit、下一字符样式、undo）。属性: value:RichText, read_only, disabled,
`typing_style_policy: RichTypingStylePolicy::{Inherit,Plain}`（默认 Inherit），font_size/font_family/
line_height/width。

工具栏走命令: `cx.rich_edit(editor_entity, RichEditCommand::*)` —— `SetAttrs(RichSpanStyle)`，
`ClearFormatting`，`InsertText`，`InsertObject(RichObject)`，`SetObject{offset,object}`，
`SetRuby(Some/None)`，`Select(range)/SelectAll`，`Undo/Redo`。剪贴板:
`cx.rich_clipboard(entity, RichClipboardCommand::{Copy,Cut,Paste,PastePlainText}, services)`。

事件: `Changed(RichText)`，`SelectionChanged{selection, attrs: RichSelectionAttrs}`（attrs.style 有
确定值或 mixed 位掩码 `RichAttrMask`；`ClearAttrs(mask)`），`ObjectActivated{id,offset}`，
`ObjectRemoved{id,offset}`，`PasteRequested`。Enter 默认插入换行 —— 要 Enter=发送，在 app 键策略
拦截（app 先看到按键）。编辑器与视图同一节点类型 → 同样的断行（同一 SetRichText 路径）；tag
`rich-text-editor` 只做纯文本初值。

### RichTextView（只读展示）
`RichTextView::new(line).font_size(20.0)`；更新: `set_component(entity, RichTextView::new(next))`
（相等不提交）或 `MutationQueue::set_rich_text(id, rich)` 直接写任意文本节点。

### 旧 Tauri HTML 内容映射
**框架没有 HTML 导入路径**（RichText 无 HTML 解析；NativeMarkdown 只吃 markdown）。映射必须
app 侧完成: 解析旧 HTML → `RichText::builder().push(区间, 样式)` 分段 + `<img>` →
`RichObject::image`。建议迁移期把 HTML→RichText 转换器放 native/ 内、一次性转换存量内容后只存
RichText（序列化走其 own 结构），不再保留 HTML 管线。

---

## 5. STATE PATTERNS

### Store derive 硬性要求（源码核对）
- `#[derive(Store)]` **只接受具名字段的 struct**；enum / tuple struct 报错 "`#[derive(Store)]`
  takes a struct with named fields"；**泛型 struct 报错** "does not take generic structs yet"
  (crates/nana-ui-view-macros/src/store.rs:36-50)。
- 生成 `<Name>StoreFields` trait（每字段一个 Subfield 访问器）—— **使用访问器的模块要 import 这个
  trait**。
- 每字段类型也要 `#[derive(Store)]`（或至少 Clone）才能继续下钻；叶子字段普通类型即可。

### 深浅追踪规则（store.rs 模块文档 1-26）
- 每条路径有 deep+shallow 两个触发器，**首次读才创建**（未读路径零成本）。
- 读值 = deep 触发；迭代列表 keys = shallow。写一条路径: 触发它自己 + 其下所有已存在路径 + 所有
  祖先的 deep。列表自身的 `push/insert/retain/swap/sort_by_key` 只触发列表 + 祖先 deep —— **行内
  容不重读，行路径保持存活**。
- 经典例（verbatim 语义）: `app.todos().at(&7).done().set(true)` 只更新一个 checkbox；
  `app.todos().push(todo)` 列表加一行，无行重读。

### Keyed 列表
`StoreList<T>`（任何 `StorePath<Value = Vec<T>>` 自动实现）: `.keyed(|item| item.id) -> KeyedList`
（key 哈希到 64-bit，**同一列表内不得碰撞**）；`.len()/.is_empty()`（只追结构）；
`.push/.insert(at,item)/.retain(keep)/.swap(a,b)/.sort_by_key`。`KeyedList`: `.at(&key) -> Item`，
`.items() -> Vec<Item>`，`.each(render_fn) -> view`，`.each_virtual`。`Item`: `.id()`，本身是
StorePath + `Readable<T>`。`Store/Subfield/Item` 都实现 `Readable + IntoProp + Writable`（值
Clone+Send 时）→ 可直接绑 `.model()` / 字段。

### provide / use_context
`provide<T>(value)`（无返回，当前作用域供给）；`use_context::<T>() -> Option<T>`（向上查找，
T: Clone）。放 app 级服务（如导航闭包、共享客户端句柄）比塞全局更 NanaUI。

### Ameya 全局 store 建议结构

```rust
#[derive(Clone, Store)]
struct AppState {
    route: Route,              // 枚举：当前页（页面切换 = route().set(..) + when(..) 切换）
    projects: Vec<Project>,    // 项目列表 → projects().keyed(|p| p.id.clone())
    library: Vec<Record>,      // 库记录（角色/设定/…… 每类一个 Vec 或 enum + tag）
    jobs: Vec<Job>,            // AI/后台任务 → jobs().keyed(|j| j.id)
    ai_settings: AiSettings,   // 嵌套 struct 也 derive(Store) → ai_settings().temperature().set(..)
    search: SearchState,       // query: Signal<String> + results: Vec<SearchHit>
}
let app = store(AppState { .. });
```
要点: 顶层一个 store；列表全部 keyed（id 稳定、同表唯一）；路由用枚举字段而不是字符串；搜索框
query 用局部 `signal`，提交时写 `app.search().query(..)`；任务进度 `jobs().at(&id).progress().set(p)`
只重绘那一行。`store_with_history(app_state, 64)` 可给需要 undo 的编辑页单独一层。

---

## 6. APP WIRING

### 两条入口
1. **手写 RuntimeProgram**（自管窗口/文档表）—— docs/guide/essentials/application.md 整段程序。
2. **ApplicationState + RuntimeApplication<State>**（普通应用推荐; UI 重写用它）。

### ApplicationState 精确签名（crates/nana-ui/src/application.rs:77-190，源码核对）
```rust
pub trait ApplicationState: Sized + 'static {
    type Message: Send + 'static;
    type Error: std::fmt::Display;
    const BUILTINS: BuiltinComponents = BuiltinComponents::Full; // 可改 Typed：只链接你创建的控件

    // 必须实现（UiReady 点：建第一屏状态，其余起 Task；build 紧随其后）
    fn initialize(context: &RuntimeProgramContext<Self::Message>) -> Result<Self, Self::Error>;
    fn build(&mut self, window: &mut ApplicationWindow,
             context: &RuntimeProgramContext<Self::Message>) -> Result<(), Self::Error>;

    // 可选覆盖（均有默认）
    fn update(&mut self, message: Self::Message,
              windows: &mut HashMap<WindowId, ApplicationWindow>,
              context: &RuntimeProgramContext<Self::Message>) -> RuntimeProgramUpdate; // 默认 no-op
    fn window_event(&mut self, event: &WindowEvent,
                    context: &RuntimeProgramContext<Self::Message>) -> RuntimeProgramUpdate;
    fn close_requested(&mut self, id: WindowId,
              windows: &mut HashMap<WindowId, ApplicationWindow>,
              context: &RuntimeProgramContext<Self::Message>) -> RuntimeProgramUpdate; // 默认关窗
    fn theme(&self) -> Arc<CompiledTheme>; // 默认 builtin_theme_arc(ThemeAppearance::Dark)
    // 其余默认: gpu_backend_policy / startup_window_material_mode / window_material_mode_for /
    // appearance_backdrop_opacity_for / input_event / animation_frame / next_wakeup / wake /
    // window_closed / prepare / presented / rebuild_gpu / output_frame / output_status /
    // host_failure / startup_takeover / startup_changed
}
```

### 保存后再关（CloseRequested 模式）
`close_requested` 默认立刻 `WindowCommand::Close(id)`。要保存询问: **返回不含
`WindowCommand::Close` 的 update**（保持开），弹确认；用户确认后在 `update` 里发
`WindowCommand::Close`（或 `RuntimeProgramUpdate::exit()` 退进程）。`window_event` 也会收到
CloseRequested，仅观察用。

### run_runtime vs NanaApplication::builder（源码核对）
- `nana_ui::run_runtime::<Program>(WindowDescriptor)`（runtime_host.rs:1042, hosted）: 开窗+建
  Program+跑循环。**不设应用路径、不开诊断**。
- `nana_ui::run_runtime_with_store::<Program>(settings, SharedStore)`（:1082）；`nana_ui::with_startup(
  StartupOptions, \|\| …)`（:1063）。
- `NanaApplication::builder(ApplicationIdentity::new(id, name, version))` → builder: `.paths(
  ApplicationPaths)`，`.diagnostics(DiagnosticsConfig)`，`.early_splash(SplashSpec)`，
  `.resource_packs(...)`，`.start() -> ApplicationSession`；**`.run::<Program>(WindowDescriptor)`**
  = start + with_startup + run_runtime（application_builder.rs:195-211）。
- Ameya 用法（照抄 examples/reactive-sfc/src/main.rs，该文件整段已核对）:
```rust
fn main() -> Result<(), nana_ui::HostedRunError> {
    let identity = ApplicationIdentity::new("studio.lilia.ameya", "Ameya", env!("CARGO_PKG_VERSION"));
    NanaApplication::builder(identity)
        .diagnostics(DiagnosticsConfig::default())
        .run::<RuntimeApplication<App>>(WindowDescriptor::new("Ameya").initial_size(1280.0, 800.0))
}
```
- ApplicationState::build 里挂树: `let document = window.document.document(); window.document
  .context_mut().mount_view_root(document, \|\| page())?;`（reactive-sfc main.rs:41-46 模式）。

### WindowDescriptor（nana-ui-platform/src/window.rs:482，源码核对）
`WindowDescriptor::new(title)`；builder: `.initial_size(w,h)`，`.minimum_size(w,h)`，
`.content_aspect_ratio(Option<f32>)`，`.icon(WindowIcon)`，`.tag(Arc<str>)`，`.persist_key(String)`，
`.system_caption(bool)`（true=保留平台标题栏；**产品窗口留 false**，NanaUI 自绘 chrome）。
pub 字段: initial_position, maximized, transparent, shadow, visible（文档初始化成功后才应用）,
always_on_top, fullscreen, focus_on_show, constrain_to_work_area, skip_taskbar, restoration_scope,
resizable, role, modal, parent, host_managed_drag, surface(WindowSurfacePreference::Auto/Platform/
Compositor)。

### 重绘
`RuntimeProgramUpdate::redraw(WindowId)` / `redraw_all()` / `exit()` / `default()` / `.merge(other)`
（runtime_host.rs:385-411）。update 里改信号 → 有待应用绑定的窗口自动请求重绘（reactivity.md）。

### 主题注册（自定义 token 时）
`ThemeDefinition`（nana-ui-core/src/theme/definition.rs:215; builder `with_id/with_display_name/
with_scrollbar/with_scrollbar_visibility` 等）描述 palette/metrics/recipes/motion/effects；
`ThemeRegistry::register(definition) -> Result<ThemeId, _>`（registry.rs:61，validate+compile），
随后 `AppContext::set_theme_definition` / `set_theme_tokens` 安装。预置
`ThemeDefinition::NANA_LIGHT / NANA_DARK`。`ThemeAppearance` 只是外观提示（Light/Dark/Custom），**不是
身份**；设置页用注册表的 `ThemeChoice` 列表，持久化只存 `theme_id`；未注册 ID 启动时
`ThemeRegistry::resolve` 回退内置 Light（`fell_back_to_light` 可诊断）。`ApplicationState::theme`
返回已编译的 `Arc<CompiledTheme>`。**控件颜色写语义角色（SemanticColorRole），不要写死亮/暗色值**。

---

## 7. HEADLESS / VALIDATION

### nana_ui::headless（hosted；crates/nana-ui/src/headless.rs，源码核对）
"A host with no native window: drives a RuntimeProgram through the same hooks, in the same order,
as the Scene host. For offscreen acceptance and tests of a whole application."

`HeadlessRuntime<P: RuntimeProgram>::new(...)` 之后:
- `.turn() -> HeadlessTurn`（一轮 present **所有**可见窗口；无事做时 flush 免费）
- `.present(id) -> Option<WorkCounters>`（跑布局）
- `.with_document(id, f)` / `.with_document_mut`（读**已 flush** 的 RuntimeDocument —— 布局/几何
  读回就在这里）
- `.dispatch(msg)` / `.update(msg)` / `.wake()` / `.input(HeadlessInput…)` / `.window_event(event)`
  / `.resize(id, logical, scale)` / `.accessibility_action` / `.apply_update(RuntimeProgramUpdate)`
- 窗口即 `HeadlessWindow{geometry, tag, visible, parent, content_zoom}`；`.window_by_tag(tag)`，
  `.take_window_commands()`，`.take_report() -> HeadlessReport{redraw, …}`，
  `set_file_dialog(HeadlessFileDialog::{Cancel,Fail,Select(paths),SelectWithFilter})`，
  `set_quiet_messages(...)`，`.context_for(id) -> RuntimeProgramContext`
- `HeadlessOptions{reduced_motion, high_contrast, reduced_transparency, text_scale}`；
  `headless_geometry(logical_size, scale) -> WindowGeometry`
- **不画像素** —— 像素是 flush 后场景上的另一步（nana-ui-devtools 负责画+读回）。窗口请求在虚拟
  窗口上成功。

### NanaUI 自己的测试怎么断言布局
crates/nana-ui-runtime/tests/: `layout_style_size.rs`（LayoutStyle 尺寸护栏），`view_stack.rs`，
`theme_layout.rs`，`deep_models.rs`，`each_virtual_focus.rs`，`smooth_scroll_cost.rs`，
`reactive_view_alloc.rs`，`view_macro.rs` + issue 回归。crates/nana-ui/tests/: `rich_text_view_api.rs`，
`chart_api.rs`，`accessibility_hidden.rs` 等。模式: **建树 → present → with_document 读节点盒 →
assert**；`LayoutViewport`/`WorkCounters` 来自 nana_ui_runtime。

### 对 native/validation/（cmake 快照验证）的结论
**可行** —— HeadlessRuntime 驱动真实程序、present() 跑真布局、with_document 读真实几何；
可以写一个 Rust 测试/二进制把节点盒导出（如 JSON）给 python/C++ 消费者，替代手搓的平行几何模型。
限制: headless **不出像素**（要像素需 nana-ui-devtools 的画+读回路径）；文本测量走真文本引擎
（nana-text），所以几何是权威值。建议: 快照断言用「几何 JSON + a11y 树」双层；视觉像素基线另走
devtools 通道（**[未验证]** devtools 具体读回 API 未核对，用前查 crates/nana-ui-devtools）。

---

## 8. SPEC RULES（应用作者硬规则）

来自 checkout AGENTS.md（66 行全读）+ docs/learn/contracts-and-invariants.md（92 行全读）+
docs/learn/authoring-paths.md（85 行全读）。

**树与状态**
1. **树挂载时建一次**。之后改信号/改那一个控件 —— 绝不在点击处理里把整页再 `mount_view_root`
   一遍（application.md 警告框原文）。树是唯一权威保留树 UiWorld；布局输出只是投影，滚动偏移
   绝不写回 Runtime 布局盒。
2. **可见动作必须接到真状态**。禁止占位路由、agent/工具说明、技术文案、无连接控件进产品 UI
   （AGENTS.md "Visible actions must be wired to real state"）。
3. 逻辑状态与呈现 overlay 分离；合成器动画不得把每帧采样写回基础样式（合同六）。

**路径与文案**
4. 路径**显示**一律 `PathText`（宽感知中省略 + 全值 tooltip；另有 `LabeledValue::path_value` /
   `SettingsRow::path_hint`）。**绝不持久化/复制/打开含换行转义的显示串**；原生对话框与可编辑路径
   字段保留真实路径。路径规范化走 `nana_ui::path`（AGENTS.md 合同）。
5. 诊断统一 `nana-diagnostics`（`event!`/`metric!`/`fault!` 静态描述符）。**帧路径零 I/O**: 禁外部
   日志 crate、`eprintln!`、字符串格式化、文件 I/O；高频数据必须是 `metric!`，不是逐帧 `event!`
   （合同七 —— 对我们: 帧/渲染热路径里不许 log，业务日志走 app 自己的诊断通道）。

**引擎与边界**
6. CSS 只有一个引擎（nana-ui-css），且只在构建期（view!/css!/.vue 编译时）解析；运行时无选择器
   匹配（合同二）。样式值用语义角色 + ThemeMetrics，别写死像素/RGB。
7. 输入路由唯一消费方是 `AppContext::route_input`（合同三）—— app 不要绕过它自算按键/指针语义。
8. 全进程单一 GPU Device/Queue、单一 WGPU 主版本；**禁止产品呈现路径 CPU 回读**、禁止第二个
   Device（合同四）。
9. 原始窗口句柄绝不进普通控件（合同五）；材质失败必须如实返回实际生效结果。辅助顶层窗口用
   `WindowDescriptor::parent` 做原生序。
10. **WebView 不是产品 UI 壳**（合同九）。窗口内嵌网页仅显式 `runtime::BrowserView` 例外。Ameya
    重写不得用 webview 兜底渲染任何产品界面。

**组件与实例化**
11. 所有控件共用单一 `ComponentRegistry` / `register_component`（合同八）—— 自定义组件也走它，
    不要发明第二套实例化 ABI。
12. 组件按语义角色表达颜色/尺寸意图；`ApplicationState::theme()` 返回注册表编译后的
    `Arc<CompiledTheme>`；设置页主题列表用 `ThemeChoice`，持久化只存 `theme_id`。

**编写路径（authoring-paths.md）**
13. 两条作者入口（Rust builders/view! 与 .vue 方言）构建期都归一为 Runtime 视图函数 + StylePatch，
    经 MutationQueue 原子提交进同一棵 UiWorld —— Ameya 用 Rust 函数式构建器即可，不引 .vue。
14. 写入只有 `UiMutation` 标准词汇（Create/Insert/Detach/SetStyle/StartAnimation/RequestFocus…）；
    slot 由 TypeBehavior::slot_assembler 在 commit 时放置，**slot 不会因每次写入被重新装配**。

**变更卫生（AGENTS.md）**
15. 公开/跨层合同变更时检查实现、消费者、docs、features、manifests、lockfiles、tests；最小根因
    改动；只为变更的行为加测试；报告精确命令与结果 —— 编译通过不等于 GPU/视觉/消费者/原生窗口
    证据。

---

## 附: 与我们现有 import 的差异清单

`nana_ui::framework::runtime::{Activate, Card, NumberInput, ScrollView, ScrollAxes, Stack, Table,
TableCell, TableRow, Text, TextArea, TextChanged, TextInput}` —— **全部真实存在**，但 canonical
写法是 `nana_ui::runtime::{…}`（framework 只是我们 lib.rs 的别名）。`nana_ui::runtime::view::
{button, column, widget}` 正确。重写时: 删 `pub use nana_ui as framework;`，全部改从
`nana_ui::runtime` / `nana_ui::runtime::view` / `nana_ui`（宿主层）/ `nana_ui_platform`
（WindowEvent/WindowId）导入。
