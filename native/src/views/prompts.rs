//! 提示词模板库：模板列表 + 编辑器（变量值 + 预览）。

use nana_ui::runtime::view::{column, dynamic, row, text, text_area, text_input, widget, IntoView};
use nana_ui::runtime::view::{Store, StorePath};
use nana_ui::runtime::{Activate, Card, TextChanged};

use crate::state::{AppMessage, AppState, AppStateStoreFields, PromptsStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    row().children((
        text("提示词模板"),
        column().children((
            list_card(app.clone()),
            editor_card(app.clone()),
            preview_card(app),
        )),
    ))
}

fn action_button(
    label: &'static str,
    message: AppMessage,
) -> nana_ui::runtime::view::El<nana_ui::runtime::Button> {
    nana_ui::runtime::view::button(label)
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(message.clone()))
}

/// 模板列表：选择后载入编辑器。
fn list_card(app: Store<AppState>) -> impl IntoView {
    let selected = app.prompts().selected_id();
    let body = dynamic(app.prompts().templates(), move |templates| {
        let selected = selected.get();
        if templates.is_empty() {
            text("（内置模板尚未载入）").into_any()
        } else {
            let rows = templates
                .iter()
                .map(|template| {
                    let id = template.id.clone();
                    let label: String = if Some(&id) == selected.as_ref() {
                        format!("● {}（{}）", template.name, template.purpose)
                    } else {
                        format!("○ {}（{}）", template.name, template.purpose)
                    };
                    row()
                        .children((
                            text(label),
                            action_button("选择", AppMessage::PromptSelect(id)),
                        ))
                        .into_any()
                })
                .collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    let new = action_button("新建模板", AppMessage::PromptNew);
    widget(Card::new().title("模板")).children(column().children((body, new)))
}

/// 编辑器：名称/用途/模板正文 + 变量值输入。
fn editor_card(app: Store<AppState>) -> impl IntoView {
    let prompts = app.prompts();
    let name = text_input()
        .placeholder("模板名称")
        .model(prompts.draft_name());
    let purpose = text_input()
        .placeholder("用途")
        .model(prompts.draft_purpose());
    let template = text_area()
        .placeholder("模板正文，变量用 {{变量名}}")
        .model(prompts.draft_template());
    let variables = dynamic(app.prompts().variable_values(), |values| {
        if values.is_empty() {
            text("此模板没有变量。").into_any()
        } else {
            let rows = values
                .iter()
                .enumerate()
                .map(|(index, (name, value))| {
                    text_input()
                        .placeholder(name.clone())
                        .value(value.clone())
                        .on_cx(move |_, event: &TextChanged, cx| {
                            cx.dispatch_program_all(AppMessage::PromptVariableValue(
                                index,
                                event.value.to_string(),
                            ))
                        })
                })
                .collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    let actions = row().children((
        action_button("保存", AppMessage::PromptSave),
        action_button("预览", AppMessage::PromptPreview),
        action_button("重置内置模板", AppMessage::PromptReset),
    ));
    let error = dynamic(app.prompts().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    let notice = dynamic(app.prompts().notice(), |notice| {
        notice.as_ref().map(|notice| text(notice.clone()))
    });
    widget(Card::new().title("编辑器")).children(column().children((
        name,
        purpose,
        template,
        text("变量值"),
        variables,
        actions,
        error,
        notice,
    )))
}

/// 预览：渲染后的提示词 + 缺失变量提示。
fn preview_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.prompts().preview(), move |preview| match preview {
        Some(preview) => {
            let missing = if preview.missing_variables.is_empty() {
                None
            } else {
                Some(text(format!(
                    "缺少变量：{}",
                    preview.missing_variables.join("、")
                )))
            };
            column()
                .children((text(preview.prompt.clone()), missing))
                .into_any()
        }
        None => text("点击「预览」查看渲染结果。").into_any(),
    });
    widget(Card::new().title("预览")).children(body)
}
