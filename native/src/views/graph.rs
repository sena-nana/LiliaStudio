//! 关系图谱页（§1.5）。GraphCanvas 同心环布局：中心 1 + 每环 12，
//! 半径百分比 18 + 12×环号（上限 40%），画布节点上限 40，超出进侧栏。
//! 聚焦对象居中（索引 0）；画布整体随过滤/聚焦变化重建（应用拥有 model）。

use ameya_core::domain::{
    axiom::Axiom,
    character::Character,
    entry::Entry,
    event::Event,
    relation::{EntityRef, Relation},
};
use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{
    button, column, computed, dynamic, empty_state, list_item, row, text, widget, AnyView,
    IntoView, StorePath,
};
use nana_ui::runtime::Stack;
use nana_ui::runtime::{Activate, Card, Select, SelectChanged, SelectOption};
use nana_ui::runtime::{GraphCanvas, GraphCanvasEvent};
use nana_ui::{
    GraphEdge, GraphEndpoint, GraphModel, GraphNode, GraphNodeId, GraphPoint, GraphPort,
    GraphPortKind, GraphPortSide, GraphSelection, GraphSize, GraphViewport,
};

use crate::state::{
    AppMessage, AppState, AppStateStoreFields, Collection, GraphStateStoreFields,
    LibraryStateStoreFields, ObjectRef, ProjectsStateStoreFields,
};

/// 画布最多绘制的节点数；超出的实体进侧栏「画布外节点」列表。
const CANVAS_NODE_CAP: usize = 40;
/// 世界坐标 = Tauri 百分比布局 ×（横向 9.6 / 纵向 5.6），默认视口下一屏可见。
const WORLD_SCALE_X: f32 = 9.6;
const WORLD_SCALE_Y: f32 = 5.6;
/// 画布高度；缩放常量在 crate::state::GRAPH_CANVAS_ZOOM（store 默认视口用）。
const CANVAS_HEIGHT: f32 = 440.0;

pub fn page(app: Store<AppState>) -> impl IntoView {
    let toolbar = toolbar(app.clone());
    let body = graph_body(app.clone());
    let error = {
        let path = app.library().error();
        dynamic(path, move |error: &Option<String>| {
            error
                .as_ref()
                .map(|message| text(format!("图谱错误：{message}")))
        })
    };
    column()
        .gap(12.0)
        .children((text("关系邻域"), toolbar, error, body))
}

fn toolbar(app: Store<AppState>) -> impl IntoView {
    let graph = app.graph();
    let library = app.library();
    let relation_types = {
        let relations = library.relations();
        let relation_filter = graph.relation_filter();
        computed(move || (relations.get(), relation_filter.get()))
    };
    let relation_select = dynamic(relation_types, move |(relations, current)| {
        let mut types: Vec<String> = relations
            .iter()
            .map(|relation| relation.relation_type.clone())
            .collect();
        types.sort();
        types.dedup();
        let mut options = vec![SelectOption::new("", "全部关系")];
        options.extend(
            types
                .iter()
                .map(|kind| SelectOption::new(kind.clone(), kind.clone())),
        );
        {
            let mut select = Select::new(select_value(current))
                .options(options)
                .label("关系类型");
            std::sync::Arc::make_mut(&mut select.style.layout).width =
                Some(nana_ui::runtime::LengthSpec::Px(220.0));
            widget(select)
        }
        .on_cx(move |_, event: &SelectChanged, cx| {
            cx.dispatch_program_all(AppMessage::GraphRelationFilterChanged(
                event.value.to_string(),
            ))
        })
    });
    let entity_types = {
        let relations = library.relations();
        let entity_filter = graph.entity_filter();
        computed(move || (relations.get(), entity_filter.get()))
    };
    let entity_select = dynamic(entity_types, move |(relations, current)| {
        let mut types: Vec<String> = relations
            .iter()
            .flat_map(|relation| {
                [
                    relation.source.entity_type.clone(),
                    relation.target.entity_type.clone(),
                ]
            })
            .collect();
        types.sort();
        types.dedup();
        let mut options = vec![SelectOption::new("", "全部实体")];
        options.extend(
            types
                .iter()
                .map(|kind| SelectOption::new(kind.clone(), entity_type_label(kind).to_string())),
        );
        {
            let mut select = Select::new(select_value(current))
                .options(options)
                .label("实体类型");
            std::sync::Arc::make_mut(&mut select.style.layout).width =
                Some(nana_ui::runtime::LengthSpec::Px(220.0));
            widget(select)
        }
        .on_cx(move |_, event: &SelectChanged, cx| {
            cx.dispatch_program_all(AppMessage::GraphEntityFilterChanged(
                event.value.to_string(),
            ))
        })
    });
    let clear_focus = {
        let path = app.graph().focus();
        dynamic(path, move |focus: &Option<String>| {
            button("清除聚焦")
                .disabled(focus.is_none())
                .on_cx(move |_, _: &Activate, cx| {
                    cx.dispatch_program_all(AppMessage::GraphFocusChanged(None))
                })
        })
    };
    let reset_view = button("重置视图")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::GraphViewportReset));
    widget(Stack::fill_row(8.0)).children((relation_select, entity_select, clear_focus, reset_view))
}

fn graph_body(app: Store<AppState>) -> impl IntoView {
    let graph = app.graph();
    let library = app.library();
    let projects = app.projects();
    let state = {
        let relations = library.relations();
        let entries = library.entries();
        let characters = library.characters();
        let events = library.events();
        let axioms = library.axioms();
        let relation_filter = graph.relation_filter();
        let entity_filter = graph.entity_filter();
        let focus = graph.focus();
        let viewport = graph.viewport();
        let active_id = projects.active_id();
        computed(move || {
            (
                active_id.get(),
                relations.get(),
                (entries.get(), characters.get(), events.get(), axioms.get()),
                relation_filter.get(),
                entity_filter.get(),
                focus.get(),
                viewport.get(),
            )
        })
    };
    let body = dynamic(state, move |payload| {
        let (
            active_id,
            relations,
            (entries, characters, events, axioms),
            relation_filter,
            entity_filter,
            focus,
            viewport,
        ) = payload;
        if active_id.is_none() {
            return empty_state("未选择项目")
                .message("先选择一个项目，再查看它的关系图谱。")
                .into_any();
        }
        let filtered: Vec<&Relation> = relations
            .iter()
            .filter(|relation| {
                relation_filter.is_empty() || relation.relation_type == *relation_filter
            })
            .filter(|relation| {
                entity_filter.is_empty()
                    || relation.source.entity_type == *entity_filter
                    || relation.target.entity_type == *entity_filter
            })
            .collect();
        if filtered.is_empty() {
            return empty_state("暂无匹配关系")
                .message("调整过滤条件，或在对象工作台里创建关系。")
                .into_any();
        }
        let entities = graph_entities(&filtered);
        let titles: Vec<(String, String)> = entities
            .iter()
            .map(|entity| {
                (
                    entity_key(entity),
                    entity_title(entity, &entries, &characters, &events, &axioms),
                )
            })
            .collect();
        let main = canvas_column(&entities, &titles, &filtered, focus.as_deref(), *viewport);
        let sidebar = sidebar_column(&filtered, &entities, &titles, focus.as_deref());
        row().gap(12.0).children((main, sidebar)).into_any()
    });
    body
}

/// 参与过滤后边集的实体，按出现顺序去重。
fn graph_entities(relations: &[&Relation]) -> Vec<EntityRef> {
    let mut entities: Vec<EntityRef> = Vec::new();
    for relation in relations {
        for entity in [&relation.source, &relation.target] {
            if !entities.iter().any(|seen| {
                seen.entity_type == entity.entity_type && seen.entity_id == entity.entity_id
            }) {
                entities.push(entity.clone());
            }
        }
    }
    entities
}

fn entity_key(entity: &EntityRef) -> String {
    format!("{}:{}", entity.entity_type, entity.entity_id)
}

/// 画布节点 id 与实体键同构（type:id）；标题查四张实体表，查不到回退 id。
fn entity_title(
    entity: &ameya_core::domain::relation::EntityRef,
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
) -> String {
    match entity.entity_type.as_str() {
        "entry" => entries
            .iter()
            .find(|item| item.id == entity.entity_id)
            .map(|item| item.title.clone())
            .unwrap_or_else(|| entity.entity_id.clone()),
        "character" => characters
            .iter()
            .find(|item| item.id == entity.entity_id)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| entity.entity_id.clone()),
        "event" => events
            .iter()
            .find(|item| item.id == entity.entity_id)
            .map(|item| item.title.clone())
            .unwrap_or_else(|| entity.entity_id.clone()),
        "axiom" => axioms
            .iter()
            .find(|item| item.id == entity.entity_id)
            .map(|item| item.subject.clone())
            .unwrap_or_else(|| entity.entity_id.clone()),
        _ => entity.entity_id.clone(),
    }
}

/// Tauri 同心环布局（src/domain/relationGraph.ts layoutPoint）：
/// 每环 12 个，半径 = min(40, 18 + 12×环号) 百分比，再按画布纵横比缩放为
/// 世界坐标。与 Tauri 的一处偏差：索引 0 取精确圆心（任务要求聚焦对象居中；
/// Tauri 原式对 ring 0 也套半径 18，中心节点悬在圆心上方）。
fn layout_point(index: usize, total: usize) -> GraphPoint {
    const CENTER_X: f32 = 50.0 * WORLD_SCALE_X;
    const CENTER_Y: f32 = 50.0 * WORLD_SCALE_Y;
    if total <= 1 || index == 0 {
        return GraphPoint::new(CENTER_X, CENTER_Y);
    }
    let ring = if index == 0 { 0 } else { (index + 11) / 12 };
    let ring_start = if ring == 0 { 0 } else { 1 + (ring - 1) * 12 };
    let ring_size = if ring == 0 {
        1
    } else {
        std::cmp::min(12, total.saturating_sub(ring_start)).max(1)
    };
    let angle = (index - ring_start) as f32 / ring_size as f32 * std::f32::consts::TAU
        - std::f32::consts::FRAC_PI_2;
    let radius = 40.0_f32.min(18.0 + ring as f32 * 12.0);
    GraphPoint::new(
        (50.0 + angle.cos() * radius) * WORLD_SCALE_X,
        (50.0 + angle.sin() * radius) * WORLD_SCALE_Y,
    )
}

/// 画布主体：GraphCanvas（默认 StandardVisual 绘制，无需宿主注册渲染器）。
/// 选中/空选 → 既有 GraphFocusChanged；平移/缩放 → GraphViewportChanged
/// （受控视口：过滤/聚焦重建画布时回传 store 中的视口，不重置用户视角）。
fn canvas_column(
    entities: &[EntityRef],
    titles: &[(String, String)],
    relations: &[&Relation],
    focus: Option<&str>,
    viewport: GraphViewport,
) -> AnyView {
    // focus 存的是 entity id（与 app.rs 既有臂/侧栏过滤一致）；
    // 画布节点 id 是 type:id 键，边界处双向换算，不改消息契约。
    let focus_key: Option<String> = focus.and_then(|entity_id| {
        entities
            .iter()
            .find(|entity| entity.entity_id == entity_id)
            .map(entity_key)
    });
    let mut ordered: Vec<(&String, &String)> =
        titles.iter().map(|item| (&item.0, &item.1)).collect();
    if let Some(focus_key) = focus_key.as_deref() {
        // 聚焦对象提到索引 0（画布中心），其余保持插入顺序（稳定排序）。
        ordered.sort_by_key(|(key, _)| key.as_str() != focus_key);
    }
    let drawn: Vec<(&String, &String)> = ordered.into_iter().take(CANVAS_NODE_CAP).collect();
    let overflow: Vec<&(String, String)> = titles
        .iter()
        .filter(|(key, _)| !drawn.iter().any(|(drawn_key, _)| *drawn_key == key))
        .collect();
    let drawn_keys: Vec<&str> = drawn.iter().map(|(key, _)| key.as_str()).collect();
    let mut nodes: Vec<GraphNode> = Vec::with_capacity(drawn.len());
    for (index, (key, title)) in drawn.iter().enumerate() {
        let position = layout_point(index, drawn.len());
        nodes.push(
            GraphNode::new(
                key.as_str(),
                title.as_str(),
                position,
                GraphSize::new(132.0, 30.0),
            )
            .with_port(GraphPort::new(
                "in",
                "关联",
                GraphPortKind::Input,
                GraphPortSide::Left,
            ))
            .with_port(GraphPort::new(
                "out",
                "关联",
                GraphPortKind::Output,
                GraphPortSide::Right,
            )),
        );
    }
    let edges: Vec<GraphEdge> = relations
        .iter()
        .filter_map(|relation| {
            let source_key = entity_key(&relation.source);
            let target_key = entity_key(&relation.target);
            let drawable = drawn_keys.contains(&source_key.as_str())
                && drawn_keys.contains(&target_key.as_str());
            drawable.then(|| {
                GraphEdge::new(
                    relation.id.clone(),
                    GraphEndpoint::new(source_key, "out"),
                    GraphEndpoint::new(target_key, "in"),
                )
            })
        })
        .collect();
    let model = GraphModel::new(nodes, edges).unwrap_or_default();
    let selection = focus_key.as_deref().and_then(|key| {
        drawn_keys
            .contains(&key)
            .then(|| GraphSelection::Node(GraphNodeId::new(key)))
    });
    let canvas = widget(
        GraphCanvas::new("ameya-graph", model)
            .label("关系图谱")
            .viewport(viewport)
            .selection(selection),
    )
    .on_cx(move |_, event: &GraphCanvasEvent, cx| {
        let entity_id = |node: &str| {
            node.split_once(':')
                .map(|(_, id)| id.to_string())
                .unwrap_or_else(|| node.to_string())
        };
        match event {
            GraphCanvasEvent::SelectionChanged(Some(GraphSelection::Node(id))) => {
                cx.dispatch_program_all(AppMessage::GraphFocusChanged(Some(entity_id(id.as_str()))))
            }
            GraphCanvasEvent::SelectionChanged(Some(GraphSelection::Port { node, .. })) => cx
                .dispatch_program_all(AppMessage::GraphFocusChanged(Some(entity_id(
                    node.as_str(),
                )))),
            GraphCanvasEvent::SelectionChanged(None) => {
                cx.dispatch_program_all(AppMessage::GraphFocusChanged(None))
            }
            GraphCanvasEvent::ViewportChanged(next) => {
                cx.dispatch_program_all(AppMessage::GraphViewportChanged(*next))
            }
            _ => {}
        }
    });
    let canvas_card = widget(Card::new().title("关系图").height(CANVAS_HEIGHT)).children(canvas);
    if overflow.is_empty() {
        return canvas_card.into_any();
    }
    column()
        .gap(12.0)
        .children((canvas_card, overflow_card(&overflow)))
        .into_any()
}

fn overflow_card(overflow: &[&(String, String)]) -> AnyView {
    let rows = overflow
        .iter()
        .map(|(key, title)| {
            let key = key.clone();
            list_item(title.clone())
                .detail(key.clone())
                .on_cx(move |_, _: &Activate, cx| {
                    cx.dispatch_program_all(AppMessage::GraphFocusChanged(Some(key.clone())))
                })
        })
        .collect::<Vec<_>>();
    widget(Card::new().title("画布外节点"))
        .children(column().gap(2.0).children(rows))
        .into_any()
}

fn sidebar_column(
    relations: &[&Relation],
    entities: &[EntityRef],
    titles: &[(String, String)],
    focus: Option<&str>,
) -> AnyView {
    let visible: Vec<&&Relation> = match focus {
        Some(focus_id) => relations
            .iter()
            .filter(|relation| {
                relation.source.entity_id == focus_id || relation.target.entity_id == focus_id
            })
            .collect(),
        None => relations.iter().collect(),
    };
    let edge_rows = visible
        .iter()
        .map(|relation| {
            let id = relation.id.clone();
            list_item(relation.relation_type.clone())
                .detail(format!(
                    "{} {} {}",
                    relation.source.entity_id,
                    if relation.directed { "→" } else { "—" },
                    relation.target.entity_id
                ))
                .on_cx(move |_, _: &Activate, cx| {
                    cx.dispatch_program_all(AppMessage::OpenObjectInWorkbench(ObjectRef {
                        kind: Collection::Relation,
                        id: id.clone(),
                    }))
                })
        })
        .collect::<Vec<_>>();
    let edges_card = widget(Card::new().title("真实关系")).children(column().gap(6.0).children((
        text(format!(
            "{} 条边 · {} 个节点（画布 {} 个）",
            visible.len(),
            entities.len(),
            titles.len().min(CANVAS_NODE_CAP)
        )),
        column().gap(2.0).children(edge_rows),
    )));
    let focus_card = match focus {
        Some(focus_id) => {
            let title = titles
                .iter()
                .find(|(key, _)| key.rsplit_once(':').map(|(_, id)| id) == Some(focus_id))
                .map(|(_, title)| title.clone())
                .unwrap_or_else(|| focus_id.to_string());
            let kind = entities
                .iter()
                .find(|entity| entity.entity_id == focus_id)
                .and_then(|entity| collection_from_entity_type(&entity.entity_type));
            let editor = if let Some(kind) = kind {
                button("编辑对象").on_cx({
                    let focus_id = focus_id.to_string();
                    move |_, _: &Activate, cx| {
                        cx.dispatch_program_all(AppMessage::OpenObjectInWorkbench(ObjectRef {
                            kind,
                            id: focus_id.clone(),
                        }))
                    }
                })
            } else {
                button("编辑对象").disabled(true)
            };
            widget(Card::new().title("聚焦节点"))
                .children(column().gap(6.0).children((text(title), editor)))
                .into_any()
        }
        None => widget(Card::new().title("聚焦节点"))
            .children(text("点击节点聚焦"))
            .into_any(),
    };
    column()
        .gap(12.0)
        .children((edges_card, focus_card))
        .into_any()
}

fn select_value(current: &str) -> Option<std::sync::Arc<str>> {
    if current.is_empty() {
        None
    } else {
        Some(std::sync::Arc::from(current))
    }
}

fn entity_type_label(entity_type: &str) -> &str {
    match entity_type {
        "entry" => "词条",
        "character" => "角色",
        "event" => "事件",
        "axiom" => "公理",
        _ => "实体",
    }
}

fn collection_from_entity_type(entity_type: &str) -> Option<Collection> {
    match entity_type {
        "entry" => Some(Collection::Entry),
        "character" => Some(Collection::Character),
        "event" => Some(Collection::Event),
        "axiom" => Some(Collection::Axiom),
        _ => None,
    }
}
