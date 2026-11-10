//! 对象工作台（§1.2）：CollectionRail / ObjectBrowser+RecordEditor / ObjectInspector
//! 三栏。选择与编辑状态全部经 AppMessage 走 app.rs 的 update，视图不触库。

use ameya_core::domain::{
    axiom::Axiom, character::Character, entry::Entry, event::Event, relation::Relation,
};
use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{
    button, chip, column, computed, dynamic, empty_state, list_item, row, text, widget, IntoView,
    StorePath,
};
use nana_ui::runtime::{
    Activate, Card, Checkbox, FormField, NumberChanged, NumberInput, Select, SelectChanged,
    SelectOption, TextArea, TextChanged, TextInput, ToggleChanged,
};

use crate::state::{
    AppMessage, AppState, AppStateStoreFields, Collection, EditorField, EditorNumberField,
    LibraryEditorDraft, LibraryStateStoreFields, ObjectRef, ProjectsStateStoreFields,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    let header = page_header(app.clone());
    let errors = error_notes(app.clone());
    let workbench = row().gap(12.0).children((
        super::pane(220.0, collection_rail(app.clone())),
        super::pane(
            400.0,
            column()
                .gap(12.0)
                .children((object_browser(app.clone()), record_editor(app.clone()))),
        ),
        super::pane(380.0, object_inspector(app)),
    ));
    column().gap(12.0).children((header, errors, workbench))
}

fn page_header(app: Store<AppState>) -> impl IntoView {
    let projects = app.projects();
    let title = {
        let active_id = projects.active_id();
        let list = projects.list();
        dynamic(
            computed(move || (active_id.get(), list.get())),
            move |(active_id, list): &(
                Option<String>,
                Vec<ameya_core::domain::project::Project>,
            )| {
                let name = active_id
                    .as_deref()
                    .and_then(|id| list.iter().find(|project| project.id == id))
                    .map(|project| project.name.clone())
                    .unwrap_or_else(|| "未选择项目".to_string());
                text(name)
            },
        )
    };
    let picker = {
        let active_id = projects.active_id();
        let list = projects.list();
        dynamic(
            computed(move || (active_id.get(), list.get())),
            move |(active_id, list): &(
                Option<String>,
                Vec<ameya_core::domain::project::Project>,
            )| {
                let options = list
                    .iter()
                    .map(|project| SelectOption::new(project.id.clone(), project.name.clone()))
                    .collect::<Vec<_>>();
                widget(
                    Select::new(active_id.clone())
                        .options(options)
                        .placeholder("选择项目")
                        .label("活动项目"),
                )
                .on_cx(move |_, event: &SelectChanged, cx| {
                    cx.dispatch_program_all(AppMessage::SelectProject(event.value.to_string()))
                })
            },
        )
    };
    let create_name = {
        let path = projects.create_name();
        dynamic(path, move |name: &String| {
            widget(
                TextInput::new(name.clone())
                    .placeholder("新项目名称")
                    .label("新项目名称"),
            )
            .on_cx(move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::ProjectNameDraftChanged(
                    event.value.to_string(),
                ))
            })
        })
    };
    let create_description = {
        let path = projects.create_description();
        dynamic(path, move |description: &String| {
            widget(
                TextInput::new(description.clone())
                    .placeholder("新项目描述")
                    .label("新项目描述"),
            )
            .on_cx(move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::ProjectDescriptionDraftChanged(
                    event.value.to_string(),
                ))
            })
        })
    };
    let create_button =
        {
            let path = projects.create_name();
            dynamic(path, move |name: &String| {
                button("新建项目").disabled(name.trim().is_empty()).on_cx(
                    move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::CreateProject),
                )
            })
        };
    column().gap(6.0).children((
        text("对象工作台"),
        title,
        picker,
        create_name,
        create_description,
        create_button,
    ))
}

fn error_notes(app: Store<AppState>) -> impl IntoView {
    let projects = app.projects();
    let library = app.library();
    let project_error = {
        let path = projects.error();
        dynamic(path, move |error: &Option<String>| {
            error
                .as_ref()
                .map(|message| text(format!("项目错误：{message}")))
        })
    };
    let library_error = {
        let path = library.error();
        dynamic(path, move |error: &Option<String>| {
            error
                .as_ref()
                .map(|message| text(format!("资料库错误：{message}")))
        })
    };
    column().gap(4.0).children((project_error, library_error))
}

fn collection_rail(app: Store<AppState>) -> impl IntoView {
    let library = app.library();
    let state = {
        let collection = library.collection();
        let entries = library.entries();
        let characters = library.characters();
        let events = library.events();
        let axioms = library.axioms();
        let relations = library.relations();
        computed(move || {
            (
                collection.get(),
                entries.get(),
                characters.get(),
                events.get(),
                axioms.get(),
                relations.get(),
            )
        })
    };
    let rail = dynamic(state, move |payload| {
        let (collection, entries, characters, events, axioms, relations) = payload;
        let count = |kind: Collection| match kind {
            Collection::Entry => entries.len(),
            Collection::Character => characters.len(),
            Collection::Event => events.len(),
            Collection::Axiom => axioms.len(),
            Collection::Relation => relations.len(),
        };
        let rows = Collection::ALL
            .iter()
            .map(|kind| {
                let selected = *collection == Some(*kind);
                list_item(format!("{}（{}）", kind.label(), count(*kind)))
                    .selected(selected)
                    .on_cx(move |_, _: &Activate, cx| {
                        cx.dispatch_program_all(AppMessage::SelectCollection(*kind))
                    })
            })
            .collect::<Vec<_>>();
        column().gap(2.0).children(rows)
    });
    widget(Card::new().title("对象")).children(rail)
}

fn object_browser(app: Store<AppState>) -> impl IntoView {
    let library = app.library();
    let state = {
        let collection = library.collection();
        let selected = library.selected();
        let entries = library.entries();
        let characters = library.characters();
        let events = library.events();
        let axioms = library.axioms();
        let relations = library.relations();
        computed(move || {
            (
                collection.get(),
                selected.get(),
                entries.get(),
                characters.get(),
                events.get(),
                axioms.get(),
                relations.get(),
            )
        })
    };
    let browser = dynamic(state, move |payload| {
        let (collection, selected, entries, characters, events, axioms, relations) =
            payload.clone();
        let Some(kind) = collection else {
            return empty_state("未选择集合")
                .message("从左侧选择一个对象集合。")
                .compact(true)
                .into_any();
        };
        let entity_count = entries.len() + characters.len() + events.len() + axioms.len();
        let create_disabled = matches!(kind, Collection::Relation) && entity_count < 2;
        let header = row().gap(8.0).children((
            text(format!(
                "{} · {} 个对象",
                kind.label(),
                collection_len(kind, &entries, &characters, &events, &axioms, &relations)
            )),
            button("新增")
                .disabled(create_disabled)
                .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::NewRecord)),
        ));
        let rows = browser_rows(
            kind,
            selected.as_ref(),
            &entries,
            &characters,
            &events,
            &axioms,
            &relations,
        );
        column().gap(6.0).children((header, rows)).into_any()
    });
    widget(Card::new().title("对象浏览")).children(browser)
}

fn collection_len(
    kind: Collection,
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
    relations: &[Relation],
) -> usize {
    match kind {
        Collection::Entry => entries.len(),
        Collection::Character => characters.len(),
        Collection::Event => events.len(),
        Collection::Axiom => axioms.len(),
        Collection::Relation => relations.len(),
    }
}

#[allow(clippy::too_many_arguments)]
fn browser_rows(
    kind: Collection,
    selected: Option<&ObjectRef>,
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
    relations: &[Relation],
) -> nana_ui::runtime::view::AnyView {
    let relation_count = |id: &str| {
        relations
            .iter()
            .filter(|relation| relation.source.entity_id == id || relation.target.entity_id == id)
            .count()
    };
    let rows: Vec<nana_ui::runtime::view::AnyView> = match kind {
        Collection::Entry => entries
            .iter()
            .map(|entry| {
                list_item(entry.title.clone())
                    .detail(format!(
                        "{} · {}",
                        if entry.entry_type.is_empty() {
                            "未分类"
                        } else {
                            &entry.entry_type
                        },
                        entry.status
                    ))
                    .note(format!(
                        "{} 关联 · {}",
                        relation_count(&entry.id),
                        updated_label(&entry.updated_at)
                    ))
                    .selected(is_selected(selected, kind, &entry.id))
                    .on_cx({
                        let id = entry.id.clone();
                        move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                                kind,
                                id: id.clone(),
                            })))
                        }
                    })
                    .into_any()
            })
            .collect(),
        Collection::Character => characters
            .iter()
            .map(|character| {
                list_item(character.name.clone())
                    .detail(if character.faction.is_empty() {
                        "未分配阵营".to_string()
                    } else {
                        character.faction.clone()
                    })
                    .note(format!(
                        "{} 关联 · {}",
                        relation_count(&character.id),
                        updated_label(&character.updated_at)
                    ))
                    .selected(is_selected(selected, kind, &character.id))
                    .on_cx({
                        let id = character.id.clone();
                        move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                                kind,
                                id: id.clone(),
                            })))
                        }
                    })
                    .into_any()
            })
            .collect(),
        Collection::Event => events
            .iter()
            .map(|event| {
                list_item(event.title.clone())
                    .detail(if event.time_label.is_empty() {
                        "未定时间".to_string()
                    } else {
                        event.time_label.clone()
                    })
                    .note(format!(
                        "{} 关联 · {}",
                        relation_count(&event.id),
                        updated_label(&event.updated_at)
                    ))
                    .selected(is_selected(selected, kind, &event.id))
                    .on_cx({
                        let id = event.id.clone();
                        move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                                kind,
                                id: id.clone(),
                            })))
                        }
                    })
                    .into_any()
            })
            .collect(),
        Collection::Axiom => axioms
            .iter()
            .map(|axiom| {
                list_item(format!("{} = {}", axiom.subject, axiom.object))
                    .detail(axiom.predicate.clone())
                    .note(format!(
                        "{} 关联 · {}",
                        relation_count(&axiom.id),
                        updated_label(&axiom.updated_at)
                    ))
                    .selected(is_selected(selected, kind, &axiom.id))
                    .on_cx({
                        let id = axiom.id.clone();
                        move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                                kind,
                                id: id.clone(),
                            })))
                        }
                    })
                    .into_any()
            })
            .collect(),
        Collection::Relation => relations
            .iter()
            .map(|relation| {
                let source = entity_label(entries, characters, events, axioms, &relation.source);
                let target = entity_label(entries, characters, events, axioms, &relation.target);
                let arrow = if relation.directed { "→" } else { "—" };
                list_item(format!("{source} {arrow} {target}"))
                    .detail(relation.relation_type.clone())
                    .note(format!(
                        "{} · {}",
                        if relation.confidence < 0.6 {
                            "低置信度"
                        } else {
                            "已确认"
                        },
                        updated_label(&relation.updated_at)
                    ))
                    .selected(is_selected(selected, kind, &relation.id))
                    .on_cx({
                        let id = relation.id.clone();
                        move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                                kind,
                                id: id.clone(),
                            })))
                        }
                    })
                    .into_any()
            })
            .collect(),
    };
    if rows.is_empty() {
        empty_state("暂无对象")
            .message("点击「新增」创建第一条记录。")
            .compact(true)
            .into_any()
    } else {
        column().gap(2.0).children(rows).into_any()
    }
}

fn is_selected(selected: Option<&ObjectRef>, kind: Collection, id: &str) -> bool {
    selected.is_some_and(|object| object.kind == kind && object.id == id)
}

fn updated_label(updated_at: &str) -> &str {
    if updated_at.is_empty() {
        "未记录"
    } else {
        updated_at
    }
}

fn record_editor(app: Store<AppState>) -> impl IntoView {
    let library = app.library();
    let state = {
        let editor = library.editor();
        let selected = library.selected();
        let presets = library.relation_presets();
        let entries = library.entries();
        let characters = library.characters();
        let events = library.events();
        let axioms = library.axioms();
        computed(move || {
            (
                editor.get(),
                selected.get(),
                presets.get(),
                entries.get(),
                characters.get(),
                events.get(),
                axioms.get(),
            )
        })
    };
    let editor = dynamic(state, move |payload| {
        let (draft, selected, presets, entries, characters, events, axioms) = payload;
        let Some(object) = selected.as_ref() else {
            return empty_state("未选择")
                .message("从对象浏览列表选择一条记录开始编辑。")
                .compact(true)
                .into_any();
        };
        let header = row().gap(8.0).children((
            text(object.kind.label()),
            button("保存")
                .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::SaveRecord)),
            button("删除").on_cx(move |_, _: &Activate, cx| {
                cx.dispatch_program_all(AppMessage::DeleteRecord)
            }),
        ));
        let fields = editor_fields(
            object.kind,
            &draft,
            &presets,
            &entries,
            &characters,
            &events,
            &axioms,
        );
        column().gap(8.0).children((header, fields)).into_any()
    });
    widget(Card::new().title("记录编辑")).children(editor)
}

#[allow(clippy::too_many_arguments)]
fn editor_fields(
    kind: Collection,
    draft: &LibraryEditorDraft,
    presets: &[String],
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
) -> nana_ui::runtime::view::AnyView {
    match kind {
        Collection::Entry => column()
            .gap(6.0)
            .children((
                text_input("标题", &draft.title, "标题", EditorField::Title),
                text_input(
                    "类型",
                    &draft.entry_type,
                    "world_rule / item / location / faction",
                    EditorField::EntryType,
                ),
                text_input("状态", &draft.status, "draft / ready…", EditorField::Status),
                text_input("标签", &draft.tags, "逗号分隔", EditorField::Tags),
                area_input("摘要", &draft.summary, "摘要", EditorField::Summary),
                area_input("正文", &draft.body, "正文", EditorField::Body),
            ))
            .into_any(),
        Collection::Character => column()
            .gap(6.0)
            .children((
                text_input("姓名", &draft.name, "姓名", EditorField::Name),
                text_input("阵营", &draft.faction, "所属阵营", EditorField::Faction),
                text_input("别名", &draft.aliases, "逗号分隔", EditorField::Aliases),
                text_input("标签", &draft.tags, "逗号分隔", EditorField::Tags),
                area_input("摘要", &draft.summary, "摘要", EditorField::Summary),
                area_input("外貌", &draft.appearance, "外貌", EditorField::Appearance),
                area_input("目标", &draft.goals, "目标", EditorField::Goals),
                area_input("动机", &draft.motivations, "动机", EditorField::Motivations),
                area_input("恐惧", &draft.fears, "恐惧", EditorField::Fears),
            ))
            .into_any(),
        Collection::Event => column()
            .gap(6.0)
            .children((
                text_input("标题", &draft.title, "标题", EditorField::Title),
                text_input(
                    "时间",
                    &draft.time_label,
                    "如 第三纪 初",
                    EditorField::TimeLabel,
                ),
                number_input(
                    "排序",
                    draft.sort_key,
                    EditorNumberField::SortKey,
                    None,
                    None,
                    1.0,
                ),
                text_input("地点", &draft.location, "地点", EditorField::Location),
                text_input(
                    "开始",
                    &draft.start_label,
                    "开始标记",
                    EditorField::StartLabel,
                ),
                text_input("结束", &draft.end_label, "结束标记", EditorField::EndLabel),
                number_input(
                    "重要度",
                    draft.importance,
                    EditorNumberField::Importance,
                    Some(0.0),
                    Some(10.0),
                    1.0,
                ),
                text_input("标签", &draft.tags, "逗号分隔", EditorField::Tags),
                area_input("描述", &draft.description, "描述", EditorField::Description),
                area_input("结果", &draft.outcome, "结果", EditorField::Outcome),
            ))
            .into_any(),
        Collection::Axiom => column()
            .gap(6.0)
            .children((
                text_input("主体", &draft.subject, "主体", EditorField::Subject),
                text_input(
                    "谓词",
                    &draft.predicate,
                    "state / 位于…",
                    EditorField::Predicate,
                ),
                text_input("对象", &draft.object, "对象", EditorField::Object),
                number_input(
                    "确定性",
                    draft.certainty,
                    EditorNumberField::Certainty,
                    Some(0.0),
                    Some(1.0),
                    0.05,
                ),
                text_input(
                    "时间范围",
                    &draft.scope_time,
                    "如 第三纪",
                    EditorField::ScopeTime,
                ),
                text_input(
                    "地点范围",
                    &draft.scope_location,
                    "如 北方",
                    EditorField::ScopeLocation,
                ),
                text_input(
                    "来源类型",
                    &draft.source_entity_type,
                    "entry / character / event",
                    EditorField::SourceEntityType,
                ),
                text_input(
                    "来源标识",
                    &draft.source_entity_id,
                    "来源对象 id",
                    EditorField::SourceEntityId,
                ),
                text_input("标签", &draft.tags, "逗号分隔", EditorField::Tags),
                area_input(
                    "自然语言",
                    &draft.natural_language,
                    "自然语言表述",
                    EditorField::NaturalLanguage,
                ),
            ))
            .into_any(),
        Collection::Relation => {
            let options = entity_options(entries, characters, events, axioms);
            let type_placeholder = presets
                .first()
                .map(String::as_str)
                .unwrap_or("参与事件 / 发生于…");
            column()
                .gap(6.0)
                .children((
                    select_input(
                        "来源",
                        &draft.relation_source,
                        options.clone(),
                        EditorField::RelationSource,
                    ),
                    select_input(
                        "目标",
                        &draft.relation_target,
                        options,
                        EditorField::RelationTarget,
                    ),
                    text_input(
                        "关系类型",
                        &draft.relation_type,
                        type_placeholder,
                        EditorField::RelationType,
                    ),
                    number_input(
                        "置信度",
                        draft.confidence,
                        EditorNumberField::Confidence,
                        Some(0.0),
                        Some(1.0),
                        0.05,
                    ),
                    widget(FormField::new("方向")).control(
                        widget(Checkbox::new("有方向", draft.directed)).on_cx(
                            move |_, event: &ToggleChanged, cx| {
                                cx.dispatch_program_all(AppMessage::EditorDirectedToggled(
                                    event.checked,
                                ))
                            },
                        ),
                    ),
                    area_input("描述", &draft.description, "描述", EditorField::Description),
                ))
                .into_any()
        }
    }
}

fn text_input(
    label: &'static str,
    value: &str,
    placeholder: &str,
    field: EditorField,
) -> impl IntoView {
    widget(FormField::new(label)).control(
        widget(TextInput::new(value.to_string()).placeholder(placeholder)).on_cx(
            move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::EditorChanged(field, event.value.to_string()))
            },
        ),
    )
}

fn area_input(
    label: &'static str,
    value: &str,
    placeholder: &str,
    field: EditorField,
) -> impl IntoView {
    widget(FormField::new(label)).control(
        widget(TextArea::new(value.to_string()).placeholder(placeholder)).on_cx(
            move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::EditorChanged(field, event.value.to_string()))
            },
        ),
    )
}

fn number_input(
    label: &'static str,
    value: f64,
    field: EditorNumberField,
    minimum: Option<f64>,
    maximum: Option<f64>,
    step: f64,
) -> impl IntoView {
    let mut input = NumberInput::new(value).step(step);
    if let Some(minimum) = minimum {
        input = input.minimum(minimum);
    }
    if let Some(maximum) = maximum {
        input = input.maximum(maximum);
    }
    widget(FormField::new(label)).control(widget(input).on_cx(
        move |_, event: &NumberChanged, cx| {
            cx.dispatch_program_all(AppMessage::EditorNumberChanged(field, event.value))
        },
    ))
}

fn select_input(
    label: &'static str,
    value: &str,
    options: Vec<SelectOption>,
    field: EditorField,
) -> impl IntoView {
    let current = (!value.is_empty()).then(|| value.to_string());
    widget(FormField::new(label)).control(
        widget(
            Select::new(current)
                .options(options)
                .placeholder("选择实体"),
        )
        .on_cx(move |_, event: &SelectChanged, cx| {
            cx.dispatch_program_all(AppMessage::EditorChanged(field, event.value.to_string()))
        }),
    )
}

fn entity_options(
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
) -> Vec<SelectOption> {
    let mut options = Vec::new();
    for entry in entries {
        options.push(SelectOption::new(
            format!("entry:{}", entry.id),
            format!("词条：{}", entry.title),
        ));
    }
    for character in characters {
        options.push(SelectOption::new(
            format!("character:{}", character.id),
            format!("角色：{}", character.name),
        ));
    }
    for event in events {
        options.push(SelectOption::new(
            format!("event:{}", event.id),
            format!("事件：{}", event.title),
        ));
    }
    for axiom in axioms {
        options.push(SelectOption::new(
            format!("axiom:{}", axiom.id),
            format!("公理：{}", axiom.subject),
        ));
    }
    options
}

fn entity_label(
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
    entity: &ameya_core::domain::relation::EntityRef,
) -> String {
    let fallback = || entity.entity_id.clone();
    match entity.entity_type.as_str() {
        "entry" => entries
            .iter()
            .find(|entry| entry.id == entity.entity_id)
            .map(|entry| entry.title.clone())
            .unwrap_or_else(fallback),
        "character" => characters
            .iter()
            .find(|character| character.id == entity.entity_id)
            .map(|character| character.name.clone())
            .unwrap_or_else(fallback),
        "event" => events
            .iter()
            .find(|event| event.id == entity.entity_id)
            .map(|event| event.title.clone())
            .unwrap_or_else(fallback),
        "axiom" => axioms
            .iter()
            .find(|axiom| axiom.id == entity.entity_id)
            .map(|axiom| axiom.subject.clone())
            .unwrap_or_else(fallback),
        _ => fallback(),
    }
}

fn object_inspector(app: Store<AppState>) -> impl IntoView {
    let library = app.library();
    let state = {
        let selected = library.selected();
        let neighborhood = library.neighborhood();
        let entries = library.entries();
        let characters = library.characters();
        let events = library.events();
        let axioms = library.axioms();
        let relations = library.relations();
        computed(move || {
            (
                selected.get(),
                neighborhood.get(),
                entries.get(),
                characters.get(),
                events.get(),
                axioms.get(),
                relations.get(),
            )
        })
    };
    let inspector = dynamic(state, move |payload| {
        let (selected, neighborhood, entries, characters, events, axioms, relations) = payload;
        let Some(object) = selected.as_ref() else {
            return empty_state("未选择")
                .message("选择一个对象后，这里显示它的邻域关系。")
                .compact(true)
                .into_any();
        };
        if object.kind == Collection::Relation {
            return relation_inspector(object, &relations, &entries, &characters, &events, &axioms);
        }
        entity_inspector(
            object,
            neighborhood.as_ref(),
            &entries,
            &characters,
            &events,
            &axioms,
        )
    });
    widget(Card::new().title("对象检视")).children(inspector)
}

fn relation_inspector(
    object: &ObjectRef,
    relations: &[Relation],
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
) -> nana_ui::runtime::view::AnyView {
    let Some(relation) = relations.iter().find(|relation| relation.id == object.id) else {
        return text("该关系已不存在").into_any();
    };
    let ends: Vec<nana_ui::runtime::view::AnyView> = [&relation.source, &relation.target]
        .iter()
        .map(|end| {
            let label = entity_label(entries, characters, events, axioms, end);
            let kind = collection_from_entity_type(&end.entity_type);
            let id = end.entity_id.clone();
            let row = if let Some(kind) = kind {
                list_item(label).on_cx(move |_, _: &Activate, cx| {
                    cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                        kind,
                        id: id.clone(),
                    })))
                })
            } else {
                list_item(label)
            };
            row.into_any()
        })
        .collect();
    column()
        .gap(6.0)
        .children((
            text(format!("置信度 {:.2}", relation.confidence)),
            text("连接"),
            column().gap(2.0).children(ends),
        ))
        .into_any()
}

fn entity_inspector(
    _object: &ObjectRef,
    neighborhood: Option<&ameya_core::services::relation_workspace::RelationNeighborhood>,
    entries: &[Entry],
    characters: &[Character],
    events: &[Event],
    axioms: &[Axiom],
) -> nana_ui::runtime::view::AnyView {
    let Some(neighborhood) = neighborhood else {
        return column()
            .gap(6.0)
            .children((
                text("邻域加载中"),
                text("选择实体后自动检视深度 1 的关系邻域。"),
            ))
            .into_any();
    };
    let center = &neighborhood.center;
    let eyebrow = text(format!(
        "{} · {} 关联",
        entity_type_label(&center.entity_type),
        neighborhood.relation_count
    ));
    let subtitle = if center.subtitle.trim().is_empty() {
        "未分类".to_string()
    } else {
        center.subtitle.trim().to_string()
    };
    let summary = if center.summary.trim().is_empty() {
        "暂无摘要".to_string()
    } else {
        center.summary.trim().to_string()
    };
    let mut sections: Vec<nana_ui::runtime::view::AnyView> = vec![
        eyebrow.into_any(),
        text(center.title.clone()).into_any(),
        text(subtitle).into_any(),
        text(summary).into_any(),
        text("已有关联").into_any(),
    ];
    if neighborhood.edges.is_empty() {
        sections.push(text("暂无已确认关系").into_any());
    } else {
        let rows = neighborhood
            .edges
            .iter()
            .map(|edge| {
                let other = if edge.source.entity_id == center.entity_id {
                    &edge.target
                } else {
                    &edge.source
                };
                let arrow = if edge.directed { "→" } else { "—" };
                let source_label = entity_label(entries, characters, events, axioms, &edge.source);
                let target_label = entity_label(entries, characters, events, axioms, &edge.target);
                let kind = collection_from_entity_type(&other.entity_type);
                let other_id = other.entity_id.clone();
                list_item(format!(
                    "{} · {source_label} {arrow} {target_label}",
                    edge.relation_type
                ))
                .on_cx(move |_, _: &Activate, cx| {
                    if let Some(kind) = kind {
                        cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                            kind,
                            id: other_id.clone(),
                        })));
                    }
                })
                .into_any()
            })
            .collect::<Vec<_>>();
        sections.push(column().gap(2.0).children(rows).into_any());
    }
    sections.push(text("待确认建议").into_any());
    if neighborhood.suggestions.is_empty() {
        sections.push(text("暂无待确认建议").into_any());
    } else {
        let rows = neighborhood
            .suggestions
            .iter()
            .map(|suggestion| {
                let suggestion = suggestion.clone();
                column()
                    .gap(2.0)
                    .children((
                        text(format!(
                            "{} · {} · {:.2}",
                            suggestion.relation_type, suggestion.strength, suggestion.confidence
                        )),
                        text(suggestion.reason.clone()),
                        button("确认").on_cx(move |_, _: &Activate, cx| {
                            cx.dispatch_program_all(AppMessage::ConfirmSuggestion(Box::new(
                                suggestion.clone(),
                            )))
                        }),
                    ))
                    .into_any()
            })
            .collect::<Vec<_>>();
        sections.push(column().gap(6.0).children(rows).into_any());
    }
    sections.push(text("缺失项").into_any());
    if neighborhood.missing.is_empty() {
        sections.push(chip("结构完整").into_any());
    } else {
        let chips = neighborhood
            .missing
            .iter()
            .map(|item| chip(item.clone()).into_any())
            .collect::<Vec<_>>();
        sections.push(row().gap(4.0).children(chips).into_any());
    }
    sections.push(text("邻域预览").into_any());
    if neighborhood.nodes.is_empty() {
        sections.push(text("暂无邻域节点").into_any());
    } else {
        let chips = neighborhood
            .nodes
            .iter()
            .map(|node| {
                let kind = collection_from_entity_type(&node.entity_type);
                let node_id = node.entity_id.clone();
                chip(node.title.clone()).on_cx(move |_, _: &Activate, cx| {
                    if let Some(kind) = kind {
                        cx.dispatch_program_all(AppMessage::SelectObject(Some(ObjectRef {
                            kind,
                            id: node_id.clone(),
                        })));
                    }
                })
            })
            .collect::<Vec<_>>();
        sections.push(row().gap(4.0).children(chips).into_any());
    }
    column().gap(6.0).children(sections).into_any()
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
