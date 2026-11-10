//! 角色成长工作台（§1.10）：左角色栏 / 中（特质状态 + 事件输入 + 预览 +
//! 来源记录）/ 右结果状态。预览与保存走 AppMessage，由 app.rs 调
//! character_growth_preview_trait_delta / create_record / workspace 重载。

use ameya_core::domain::character::Character;
use ameya_core::domain::character_trait_delta::CharacterTraitDeltaRecord;
use ameya_core::domain::event::Event;
use ameya_core::services::character_growth::CharacterTraitState;
use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{
    button, column, computed, dynamic, empty_state, list_item, row, spinner, status_badge, text,
    widget, IntoView, StorePath,
};
use nana_ui::runtime::Stack;
use nana_ui::runtime::{
    Activate, Card, Select, SelectChanged, SelectOption, StatusTone as BadgeTone, TextArea,
    TextChanged, TextInput,
};

use crate::state::{
    AppMessage, AppState, AppStateStoreFields, GrowthStateStoreFields, LibraryStateStoreFields,
    ProjectsStateStoreFields,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    let ladder = {
        let projects = app.projects();
        let growth = app.growth();
        let library = app.library();
        let active_id = projects.active_id();
        let workspace = growth.workspace();
        let characters = library.characters();
        let events = library.events();
        computed(move || {
            (
                active_id.get().is_some(),
                workspace.get().is_some(),
                characters.get().is_empty(),
                events.get().is_empty(),
            )
        })
    };
    let ladder = dynamic(
        ladder,
        move |&(has_project, has_workspace, no_characters, no_events)| {
            if !has_project {
                return empty_state("未选择项目")
                    .message("先选择一个项目，再打开成长工作台。")
                    .into_any();
            }
            if !has_workspace {
                return empty_state("成长工作台未加载")
                    .message("读取项目内的角色、事件和成长记录。")
                    .action(button("加载成长工作台").on_cx(move |_, _: &Activate, cx| {
                        cx.dispatch_program_all(AppMessage::LoadGrowthWorkspace)
                    }))
                    .into_any();
            }
            if no_characters {
                return empty_state("暂无角色")
                    .message("先在资料库里创建角色，再记录他们的成长。")
                    .into_any();
            }
            if no_events {
                return empty_state("暂无事件")
                    .message("先创建项目事件，成长记录需要一个来源事件。")
                    .into_any();
            }
            text("").into_any()
        },
    );
    let characters_pane = super::pane(200.0, characters_pane(app.clone()));
    let middle = super::pane(
        500.0,
        column().gap(12.0).children((
            trait_pane(app.clone()),
            form_pane(app.clone()),
            preview_pane(app.clone()),
            records_pane(app.clone()),
        )),
    );
    let right = super::pane(320.0, status_pane(app));
    column().gap(12.0).children((
        text("成长工作台"),
        ladder,
        row().gap(12.0).children((characters_pane, middle, right)),
    ))
}

fn characters_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let library = app.library();
    let state = {
        let characters = library.characters();
        let workspace = growth.workspace();
        let selected = growth.selected_character();
        computed(move || (characters.get(), workspace.get(), selected.get()))
    };
    dynamic(state, move |(characters, workspace, selected)| {
        let mut sorted: Vec<&Character> = characters.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        let states = workspace.as_ref().map(|workspace| &workspace.states);
        let rows = sorted
            .iter()
            .map(|character| {
                let record_count = states
                    .and_then(|states| states.get(&character.id))
                    .map(|state| state.sources.len())
                    .unwrap_or(0);
                let id = character.id.clone();
                list_item(character.name.clone())
                    .detail(if character.faction.is_empty() {
                        "未分配阵营".to_string()
                    } else {
                        character.faction.clone()
                    })
                    .note(format!("记录 {record_count}"))
                    .selected(selected.as_deref() == Some(character.id.as_str()))
                    .on_cx(move |_, _: &Activate, cx| {
                        cx.dispatch_program_all(AppMessage::GrowthCharacterSelected(Some(
                            id.clone(),
                        )))
                    })
            })
            .collect::<Vec<_>>();
        widget(Card::new().title(format!("角色 · {} 个", characters.len())))
            .children(column().gap(2.0).children(rows))
            .into_any()
    })
    .into_any()
}

fn selected_name(characters: &[Character], selected: Option<&String>) -> String {
    selected
        .as_deref()
        .and_then(|id| {
            characters
                .iter()
                .find(|character| character.id.as_str() == id)
        })
        .map(|character| character.name.clone())
        .unwrap_or_else(|| "未选择角色".to_string())
}

fn trait_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let library = app.library();
    let state = {
        let workspace = growth.workspace();
        let selected = growth.selected_character();
        let characters = library.characters();
        computed(move || (workspace.get(), selected.get(), characters.get()))
    };
    dynamic(state, move |(workspace, selected, characters)| {
        let traits = selected
            .as_deref()
            .and_then(|id| {
                workspace
                    .as_ref()
                    .and_then(|workspace| workspace.states.get(id))
            })
            .map(|state: &CharacterTraitState| &state.values);
        match traits {
            Some(values) if !values.is_empty() => {
                let rows = values
                    .iter()
                    .map(|(name, value)| list_item(name.clone()).detail(format!("{:+.2}", value)))
                    .collect::<Vec<_>>();
                widget(Card::new().title("当前特质状态"))
                    .children((
                        text(selected_name(&characters, selected.as_ref())),
                        column().gap(2.0).children(rows),
                    ))
                    .into_any()
            }
            _ => widget(Card::new().title("当前特质状态"))
                .children((
                    text(selected_name(&characters, selected.as_ref())),
                    text("暂无特质记录"),
                ))
                .into_any(),
        }
        .into_any()
    })
    .into_any()
}

fn form_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let library = app.library();
    let event_state = {
        let events = library.events();
        let source_event = growth.source_event();
        computed(move || (events.get(), source_event.get()))
    };
    let event_select = dynamic(event_state, move |(events, source_event)| {
        let mut sorted: Vec<&Event> = events.iter().collect();
        sorted.sort_by_key(|event| event.sort_key);
        let mut options = vec![SelectOption::new("", "请选择事件")];
        options.extend(sorted.iter().map(|event| {
            let time = if event.time_label.is_empty() {
                "未定时间"
            } else {
                event.time_label.as_str()
            };
            SelectOption::new(event.id.clone(), format!("{} · {}", event.title, time))
        }));
        widget(
            Select::new(source_event.clone())
                .options(options)
                .label("来源事件"),
        )
        .on_cx(move |_, event: &SelectChanged, cx| {
            let value = event.value.to_string();
            let value = (!value.is_empty()).then_some(value);
            cx.dispatch_program_all(AppMessage::GrowthSourceEventSelected(value))
        })
    });
    let trait_name = {
        let path = growth.trait_name();
        dynamic(path, move |name: &String| {
            widget(
                TextInput::new(name.clone())
                    .placeholder("例如 responsibility")
                    .label("Trait 名称"),
            )
            .on_cx(move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::GrowthTraitNameChanged(event.value.to_string()))
            })
        })
    };
    let delta = {
        let path = growth.delta_text();
        dynamic(path, move |value: &String| {
            widget(
                TextInput::new(value.clone())
                    .placeholder("例如 0.35")
                    .label("Delta"),
            )
            .on_cx(move |_, event: &TextChanged, cx| {
                cx.dispatch_program_all(AppMessage::GrowthDeltaChanged(event.value.to_string()))
            })
        })
    };
    let reason = {
        let path = growth.reason();
        dynamic(path, move |value: &String| {
            widget(TextArea::new(value.clone()).placeholder("描述事件如何影响角色 trait")).on_cx(
                move |_, event: &TextChanged, cx| {
                    cx.dispatch_program_all(AppMessage::GrowthReasonChanged(
                        event.value.to_string(),
                    ))
                },
            )
        })
    };
    let button_state = {
        let source_event = growth.source_event();
        let trait_name = growth.trait_name();
        let delta_text = growth.delta_text();
        let reason = growth.reason();
        let previewing = growth.previewing();
        let saving = growth.saving();
        let preview = growth.preview();
        computed(move || {
            (
                source_event.get(),
                trait_name.get(),
                delta_text.get(),
                reason.get(),
                previewing.get(),
                saving.get(),
                preview.get(),
            )
        })
    };
    let buttons = dynamic(
        button_state,
        move |(source_event, trait_name, delta_text, reason, previewing, saving, preview)| {
            let delta_ok = delta_text.trim().parse::<f32>().is_ok();
            let can_submit = source_event.is_some()
                && !trait_name.trim().is_empty()
                && delta_ok
                && !reason.trim().is_empty();
            widget(Stack::fill_row(8.0)).children((
                button(if *previewing {
                    "预览中"
                } else {
                    "预览变化"
                })
                .disabled(!can_submit || *previewing)
                .on_cx(move |_, _: &Activate, cx| {
                    cx.dispatch_program_all(AppMessage::GrowthPreview)
                }),
                button(if *saving { "保存中" } else { "保存记录" })
                    .disabled(preview.is_none() || *saving)
                    .on_cx(move |_, _: &Activate, cx| {
                        cx.dispatch_program_all(AppMessage::GrowthSave)
                    }),
            ))
        },
    );
    widget(Card::new().title("来源事件输入"))
        .children(column().gap(6.0).children((
            text("选择项目内事件，填写本次成长影响。"),
            event_select,
            trait_name,
            delta,
            reason,
            buttons,
        )))
        .into_any()
}

fn preview_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let state = {
        let preview = growth.preview();
        let previewing = growth.previewing();
        computed(move || (preview.get(), previewing.get()))
    };
    dynamic(state, move |(preview, previewing)| {
        if *previewing {
            return widget(Card::new().title("预览"))
                .children(row().gap(6.0).children((spinner(), text("预览中"))))
                .into_any();
        }
        let Some(preview) = preview else {
            return widget(Card::new().title("预览"))
                .children(text("尚未生成预览"))
                .into_any();
        };
        let rows = preview
            .values
            .iter()
            .map(|(name, value)| list_item(name.clone()).detail(format!("{:+.2}", value)))
            .collect::<Vec<_>>();
        widget(Card::new().title("预览"))
            .children(column().gap(2.0).children(rows))
            .into_any()
    })
    .into_any()
}

fn records_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let library = app.library();
    let both = {
        let workspace = growth.workspace();
        let events = library.events();
        computed(move || (workspace.get(), events.get()))
    };
    dynamic(both, move |(workspace, events)| {
        let Some(workspace) = workspace.as_ref() else {
            return widget(Card::new().title("来源记录"))
                .children(text("请先加载成长工作台"))
                .into_any();
        };
        let mut ordered: Vec<&CharacterTraitDeltaRecord> = workspace.records.iter().collect();
        ordered.reverse();
        let rows = ordered
            .iter()
            .map(|record| {
                let event = events
                    .iter()
                    .find(|event: &&Event| event.id == record.source_event_id);
                let event_title = event
                    .map(|event| event.title.clone())
                    .unwrap_or_else(|| "未定事件".to_string());
                let time_label = event
                    .map(|event| {
                        if event.time_label.is_empty() {
                            "未定时间".to_string()
                        } else {
                            event.time_label.clone()
                        }
                    })
                    .unwrap_or_else(|| "未定时间".to_string());
                list_item(format!(
                    "{event_title} · {} {:+.2}",
                    record.trait_name, record.delta
                ))
                .detail(if record.reason.is_empty() {
                    "未填写原因".to_string()
                } else {
                    record.reason.clone()
                })
                .note(format!("{time_label} · {}", record.created_at))
            })
            .collect::<Vec<_>>();
        if rows.is_empty() {
            widget(Card::new().title("来源记录"))
                .children(text("暂无来源记录"))
                .into_any()
        } else {
            widget(Card::new().title("来源记录"))
                .children(column().gap(2.0).children(rows))
                .into_any()
        }
    })
    .into_any()
}

fn status_pane(app: Store<AppState>) -> nana_ui::runtime::view::AnyView {
    let growth = app.growth();
    let library = app.library();
    let state = {
        let selected_character = growth.selected_character();
        let source_event = growth.source_event();
        let trait_name = growth.trait_name();
        let delta_text = growth.delta_text();
        let reason = growth.reason();
        let previewing = growth.previewing();
        let saving = growth.saving();
        let preview = growth.preview();
        let error = growth.error();
        let notice = growth.notice();
        let characters = library.characters();
        computed(move || {
            (
                selected_character.get(),
                source_event.get(),
                trait_name.get(),
                delta_text.get(),
                reason.get(),
                previewing.get(),
                saving.get(),
                preview.get(),
                error.get(),
                notice.get(),
                characters.get(),
            )
        })
    };
    dynamic(
        state,
        move |(
            selected_character,
            source_event,
            trait_name,
            delta_text,
            reason,
            previewing,
            saving,
            preview,
            error,
            notice,
            characters,
        )| {
            let status = if error.is_some() {
                "失败"
            } else if *saving {
                "保存中"
            } else if *previewing {
                "预览中"
            } else if notice.is_some() {
                "已保存"
            } else if preview.is_some() {
                "已生成预览"
            } else {
                "空闲"
            };
            let delta_ok = delta_text.trim().parse::<f32>().is_ok();
            let checks = row().gap(4.0).children((
                status_badge(if source_event.is_some() {
                    "事件 ✓"
                } else {
                    "事件"
                })
                .tone(if source_event.is_some() {
                    BadgeTone::Success
                } else {
                    BadgeTone::Neutral
                }),
                status_badge(if !trait_name.trim().is_empty() {
                    "Trait ✓"
                } else {
                    "Trait"
                })
                .tone(if trait_name.trim().is_empty() {
                    BadgeTone::Neutral
                } else {
                    BadgeTone::Success
                }),
                status_badge(if delta_ok { "Delta ✓" } else { "Delta" }).tone(if delta_ok {
                    BadgeTone::Success
                } else {
                    BadgeTone::Neutral
                }),
                status_badge(if !reason.trim().is_empty() {
                    "原因 ✓"
                } else {
                    "原因"
                })
                .tone(if reason.trim().is_empty() {
                    BadgeTone::Neutral
                } else {
                    BadgeTone::Success
                }),
            ));
            let notice_line = error
                .clone()
                .or(notice.clone())
                .map(|message| text(message));
            widget(Card::new().title("结果状态"))
                .children(column().gap(6.0).children((
                    text(format!("状态：{status}")),
                    text(format!(
                        "当前角色：{}",
                        selected_name(&characters, selected_character.as_ref())
                    )),
                    checks,
                    notice_line,
                    text("提示：第一版不会校验该事件是否真的包含当前角色参与记录。"),
                )))
                .into_any()
        },
    )
    .into_any()
}
