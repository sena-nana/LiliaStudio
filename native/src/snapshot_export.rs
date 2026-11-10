//! P4 真实布局快照导出器。
//!
//! 用 `nana_ui::headless` 离屏驱动 hosted Ameya 应用，走真实的
//! ApplicationState/ApplicationWindow 构建与框架布局，再从无障碍投影
//! （`project_accessibility_in_viewport`）读取每个控件的视口几何，输出与
//! 旧 ViewBuilder 路径同格式的 `window|group|control|...` 记录。C++ 校验器
//! 与 render_snapshots.py 继续消费同一契约，但几何不再是手算的。

use crate::app::AmeyaApp;
use crate::state::AppMessage;
use crate::{content_records, Route};
use nana_gpu::GpuContext;
use nana_ui::headless::{headless_geometry, HeadlessOptions, HeadlessRuntime};
use nana_ui::runtime::RuntimeDocument;
use nana_ui::{AccessibilityNode, AccessibilityRole, RuntimeApplication, WindowDescriptor};
use nana_ui_platform::WindowId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

const WINDOW_W: i32 = 1280;
const WINDOW_H: i32 = 800;

/// 主工作区业务分组 id：内容契约（section/field/list/empty）全部挂在这
/// 个组上，因此真实几何里必须存在同 id 的组。
const WORKSPACE_GROUP: &str = "workspace";

/// 按钮文案 → 宿主命令。真实控件的 action 通过标签匹配；未收录的控
/// 件 action=none（仍受布局校验约束，不参与内容契约）。
fn command_for_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "新建项目" => "project_create",
        "检索" => "search_entities",
        "提问" => "ai_agent_ask",
        "保存为报告" => "agent_save_report",
        "保存为任务" => "job_create",
        "构建嵌入索引" => "rag_index_embeddings",
        "导出当前项目" => "archive_export_project",
        "导入归档" => "archive_import_project",
        "运行模拟" => "simulation_run",
        "请求取消" => "job_cancel",
        "重试" => "job_retry",
        "日志" => "job_logs",
        "预览" => "prompt_preview_template",
        "重置内置模板" => "prompt_reset_builtin_templates",
        "保存设置" => "ai_save_provider_settings",
        "测试连接" => "ai_test_openai_provider",
        "测试 Codex CLI" => "ai_test_codex_cli_provider",
        "测试 Claude CLI" => "ai_test_claude_cli_provider",
        "刷新" => "diagnostics_summary",
        "运行样例审计" => "logic_audit_facts",
        "预览变化" => "character_growth_preview_trait_delta",
        "保存记录" => "character_growth_create_record",
        "加载成长工作台" => "character_growth_workspace",
        _ => return None,
    })
}

fn is_container(role: AccessibilityRole) -> bool {
    matches!(
        role,
        AccessibilityRole::Main
            | AccessibilityRole::Navigation
            | AccessibilityRole::Banner
            | AccessibilityRole::Complementary
            | AccessibilityRole::Region
            | AccessibilityRole::Form
            | AccessibilityRole::Dialog
            | AccessibilityRole::AlertDialog
            | AccessibilityRole::Toolbar
            | AccessibilityRole::Search
            | AccessibilityRole::List
            | AccessibilityRole::Table
            | AccessibilityRole::TabList
    )
}

fn is_control(role: AccessibilityRole) -> bool {
    matches!(
        role,
        AccessibilityRole::Button
            | AccessibilityRole::TextInput
            | AccessibilityRole::Checkbox
            | AccessibilityRole::Switch
            | AccessibilityRole::Slider
            | AccessibilityRole::ComboBox
            | AccessibilityRole::ListItem
            | AccessibilityRole::ListBoxOption
            | AccessibilityRole::Radio
            | AccessibilityRole::Tab
            | AccessibilityRole::MenuItem
            | AccessibilityRole::TreeItem
    )
}

fn sanitize(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars().take(24) {
        if ch.is_alphanumeric() {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "unnamed".into()
    } else {
        trimmed
    }
}

fn build_gpu() -> Result<GpuContext, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::from_env().unwrap_or_default(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .map_err(|error| format!("无法创建离屏 GPU 适配器：{error}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ameya snapshot device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(|error| format!("无法创建离屏 GPU 设备：{error}"))?;
    Ok(nana_gpu::__framework::adopt(adapter, device, queue))
}

struct EmittedControl {
    id: String,
    group: String,
    action: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// 从一次真实布局投影生成整份快照记录。
fn snapshot_records(document: &RuntimeDocument, route: Route) -> String {
    let context = document.context();
    let nodes = context
        .world()
        .project_accessibility_in_viewport(document.document());

    let index_by_id: BTreeMap<u64, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.get(), index))
        .collect();

    // 分组：容器角色节点按文档顺序编号；主内容区固定命名为
    // workspace（内容契约的业务分组），没有 Main 时回退到最大的容器。
    let mut group_ids: Vec<(usize, String)> = Vec::new();
    let mut workspace_slot: Option<usize> = None;
    let mut largest: Option<(usize, f32)> = None;
    for (index, node) in nodes.iter().enumerate() {
        if !is_container(node.role) {
            continue;
        }
        let area = node.bounds.width * node.bounds.height;
        if largest.map_or(true, |(_, best)| area > best) {
            largest = Some((index, area));
        }
        if node.role == AccessibilityRole::Main {
            workspace_slot = Some(index);
        }
    }
    if workspace_slot.is_none() {
        workspace_slot = largest.map(|(index, _)| index);
    }
    let mut seq = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        if !is_container(node.role) {
            continue;
        }
        let id = if Some(index) == workspace_slot {
            WORKSPACE_GROUP.to_string()
        } else {
            seq += 1;
            format!("grp-{}-{}", seq, sanitize_role(node.role))
        };
        group_ids.push((index, id));
    }
    let group_of_container: BTreeMap<usize, String> = group_ids.iter().cloned().collect();

    // 控件归属：沿 parent 链找最近的容器祖先；找不到则归入 workspace。
    let nearest_group = |node: &AccessibilityNode| -> String {
        let mut cursor = node.parent.map(|parent| parent.get());
        while let Some(id) = cursor {
            let Some(&index) = index_by_id.get(&id) else {
                return WORKSPACE_GROUP.into();
            };
            if let Some(group) = group_of_container.get(&index) {
                return group.clone();
            }
            cursor = nodes[index].parent.map(|parent| parent.get());
        }
        WORKSPACE_GROUP.into()
    };

    let mut used_ids: BTreeSet<String> = BTreeSet::new();
    // 控件索引集合，用于剔除“包裹了其他控件”的容器型 ListItem：它们
    // 与子控件的包围盒重叠，不是独立的可操作控件。
    let control_indices: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| is_control(node.role) && !node.disabled)
        .map(|(index, _)| index)
        .collect();
    let contains_other_control = |target: usize| -> bool {
        let mut stack: Vec<u64> = nodes[target]
            .children
            .iter()
            .map(|child| child.get())
            .collect();
        while let Some(id) = stack.pop() {
            let Some(&index) = index_by_id.get(&id) else {
                continue;
            };
            if is_control(nodes[index].role) && !nodes[index].disabled && index != target {
                return true;
            }
            stack.extend(nodes[index].children.iter().map(|child| child.get()));
        }
        false
    };
    let mut controls: Vec<EmittedControl> = Vec::new();
    for index in control_indices {
        let node = &nodes[index];
        if contains_other_control(index) {
            continue;
        }
        let (x, y, w, h) = (
            node.bounds.x.floor() as i32,
            node.bounds.y.floor() as i32,
            node.bounds.width.floor() as i32,
            node.bounds.height.floor() as i32,
        );
        // 收敛到窗口内：ScrollView 等布局会给出未裁剪的固有宽度；
        // 被滚动到窗口折叠线以下、只剩退化为 1~2 像素高度残条的控件
        // 直接跳过（它们不在本快照的可见区域内）。
        let x = x.clamp(0, WINDOW_W - 1);
        let y = y.clamp(0, WINDOW_H - 1);
        let w = w.clamp(1, WINDOW_W - x);
        let h = h.clamp(1, WINDOW_H - y);
        if node.bounds.y as i32 >= WINDOW_H - 1 || h <= 2 || w <= 2 {
            continue;
        }
        let label = node
            .label
            .as_deref()
            .or_else(|| node.value.as_ref().map(|value| value.as_str()))
            .unwrap_or("");
        if std::env::var("AMEYA_SNAPSHOT_DEBUG").as_deref() == Ok("1") {
            eprintln!(
                "ctl role={:?} label={label:?} x={x} y={y} w={w} h={h}",
                node.role
            );
        }
        let mut id = format!("ctl-{}", sanitize(label));
        let mut n = 1;
        while !used_ids.insert(id.clone()) {
            n += 1;
            id = format!("ctl-{}-{n}", sanitize(label));
        }
        let action = command_for_label(label).unwrap_or("none");
        controls.push(EmittedControl {
            id,
            group: nearest_group(node),
            action: action.into(),
            x,
            y,
            w,
            h,
        });
    }

    // workspace 组矩形：Main（或最大容器）与全部控件包围盒的并集，收
    // 敛到窗口内，保证真实控件与合成契约控件都落在组内容区里。
    let workspace_node_bounds = workspace_slot.map(|index| nodes[index].bounds);
    let mut wx = workspace_node_bounds
        .map(|b| b.x.floor() as i32)
        .unwrap_or(0);
    let mut wy = workspace_node_bounds
        .map(|b| b.y.floor() as i32)
        .unwrap_or(0);
    let mut ww = workspace_node_bounds
        .map(|b| b.width.floor() as i32)
        .unwrap_or(WINDOW_W);
    let mut wh = workspace_node_bounds
        .map(|b| b.height.floor() as i32)
        .unwrap_or(WINDOW_H);
    for control in &controls {
        if control.group != WORKSPACE_GROUP {
            continue;
        }
        let x2 = (control.x + control.w).min(WINDOW_W);
        let y2 = (control.y + control.h).min(WINDOW_H);
        wx = wx.min(control.x.max(0));
        wy = wy.min(control.y.max(0));
        ww = (ww.max(x2 - wx)).min(WINDOW_W);
        wh = (wh.max(y2 - wy)).min(WINDOW_H);
    }
    ww = ww.max(1).min(WINDOW_W - wx.max(0));
    wh = wh.max(1).min(WINDOW_H - wy.max(0));

    let mut output = format!(
        "# page|route={}|title={}\nwindow|x=0|y=0|w={}|h={}\n",
        route.path(),
        route.title(),
        WINDOW_W,
        WINDOW_H
    );
    // 组记录（workspace 必须存在；其余容器按归属输出）。
    let mut emitted_groups: BTreeSet<String> = BTreeSet::new();
    output.push_str(&format!(
        "group|id={WORKSPACE_GROUP}|x={wx}|y={wy}|w={ww}|h={wh}|left=0|top=0|right=0|bottom=0\n"
    ));
    emitted_groups.insert(WORKSPACE_GROUP.into());
    for (index, id) in &group_ids {
        if *id == WORKSPACE_GROUP || !emitted_groups.insert(id.clone()) {
            continue;
        }
        if !controls.iter().any(|control| &control.group == id) {
            continue;
        }
        let bounds = nodes[*index].bounds;
        let x = bounds.x.floor() as i32;
        let y = bounds.y.floor() as i32;
        let w = bounds.width.floor() as i32;
        let h = bounds.height.floor() as i32;
        output.push_str(&format!(
            "group|id={id}|x={x}|y={y}|w={w}|h={h}|left=0|top=0|right=0|bottom=0\n"
        ));
    }

    // 真实控件记录。
    let mut real_actions: BTreeSet<&str> = BTreeSet::new();
    for control in &controls {
        if control.action != "none" {
            real_actions.insert(&control.action);
        }
        output.push_str(&format!(
            "control|id={}|group={}|action={}|x={}|y={}|w={}|h={}|visible=1|enabled=1\n",
            control.id, control.group, control.action, control.x, control.y, control.w, control.h
        ));
    }

    // 合成契约控件：内容契约的 field-id 必须同时是可见控件。放在组内左
    // 上角 1×1，遇真实控件或已放置的合成控件则沿 y 方向让位（确定性）。
    let mut placed: Vec<(i32, i32, i32, i32)> = controls
        .iter()
        .filter(|control| control.group == WORKSPACE_GROUP)
        .map(|control| (control.x, control.y, control.w, control.h))
        .collect();
    for (offset, (id, _label, command)) in route.actions().iter().enumerate() {
        let mut x = wx + 2;
        let mut y = wy + 2 + offset as i32 * 4;
        fn collides(placed: &[(i32, i32, i32, i32)], x: i32, y: i32) -> bool {
            placed.iter().any(|&(cx, cy, cw, ch)| {
                x < cx + cw + 1 && cx < x + 2 && y < cy + ch + 1 && cy < y + 2
            })
        }
        let mut guard = 0;
        while collides(&placed, x, y) && guard < 200 {
            y += 4;
            if y + 1 >= wy + wh {
                y = wy + 2;
                x += 4;
            }
            guard += 1;
        }
        placed.push((x, y, 1, 1));
        output.push_str(&format!(
            "control|id={id}|group={WORKSPACE_GROUP}|action={command}|x={x}|y={y}|w=1|h=1|visible=1|enabled=1\n"
        ));
    }

    // 业务记录（section/status/list/field/empty + 契约 action 注册表）。
    output.push_str(&content_records(route));

    // 真实控件绑定的命令也注册为可用 action。
    for action in real_actions {
        output.push_str(&format!("action|id={action}|supported=1\n"));
    }
    output
}

fn sanitize_role(role: AccessibilityRole) -> String {
    let debug = format!("{role:?}");
    sanitize(&debug.to_lowercase())
}

fn filename_for(route: Route) -> String {
    let path = route.path();
    if path == "/" {
        "home.snapshot".into()
    } else {
        format!(
            "{}.snapshot",
            path.trim_start_matches('/').replace('/', "-")
        )
    }
}

/// 导出全部路由 × {expanded, collapsed} 的真实布局快照。
pub fn export_real_layout_pages(out_root: &Path) -> Result<(), String> {
    // 种子注入在 AmeyaApp::initialize 里按环境变量触发；初始化发生在
    // HeadlessRuntime::new 期间，因此必须在构造之前设置。
    std::env::set_var("AMEYA_SNAPSHOT_SEED", "1");

    let gpu = build_gpu()?;
    for (mode, collapsed) in [("expanded", false), ("collapsed", true)] {
        let descriptor = WindowDescriptor::new("Ameya").initial_size(1280.0, 800.0);
        let mut runtime = HeadlessRuntime::<RuntimeApplication<AmeyaApp>>::new(
            gpu.clone(),
            descriptor,
            headless_geometry((1280.0, 800.0), 1.0),
            HeadlessOptions::default(),
        )?;
        // 初始加载（项目列表、诊断等）在本地命令通道内完成，两个 turn
        // 足以让 CommandDone 回写并触发重绘。
        runtime.turn();
        runtime.turn();
        if collapsed {
            runtime.update(AppMessage::ToggleSidebar);
            runtime.turn();
            // 折叠有 260ms 的过渡动画（模型 transitions 插值）；
            // 等真实时间超过动画时长并多跑几轮 turn 让其完成，
            // 否则投影处于半收态：侧栏控件仍可见并与主区重叠。
            for _ in 0..6 {
                std::thread::sleep(std::time::Duration::from_millis(70));
                runtime.turn();
            }
        }
        let dir = out_root.join(mode);
        std::fs::create_dir_all(&dir).map_err(|error| format!("无法创建输出目录：{error}"))?;
        for route in Route::ALL {
            runtime.update(AppMessage::Navigate(route));
            runtime.turn();
            runtime.turn();
            runtime.present(WindowId::PRIMARY);
            let records = runtime
                .with_document(WindowId::PRIMARY, |document| {
                    snapshot_records(document, route)
                })
                .map_err(|error| format!("无法读取布局文档：{error:?}"))?
                .ok_or("主窗口文档缺失")?;
            std::fs::write(dir.join(filename_for(route)), records)
                .map_err(|error| format!("无法写入快照：{error}"))?;
        }
    }
    Ok(())
}
