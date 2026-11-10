//! 智能体问答：提问 → 引用检索 → 回答；支持保存为报告（词条）。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, text_area, widget, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{
    AgentStateStoreFields, AppMessage, AppState, AppStateStoreFields, ProjectsStateStoreFields,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((text("智能体"), ask_card(app.clone()), answer_card(app)))
}

/// 提问区：问题输入 + 提问按钮；需要已选项目。
fn ask_card(app: Store<AppState>) -> impl IntoView {
    let agent = app.agent();
    let input = text_area()
        .placeholder("向项目资料提问，例如：北方的粮食危机会如何影响城墙守军？")
        .model(agent.question());
    let ask = nana_ui::runtime::view::button("提问")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::AgentAsk));
    let state_line = dynamic(app.agent().asking(), move |asking| {
        if *asking {
            text("正在检索引用并生成回答…")
        } else {
            text("")
        }
    });
    let project_line = dynamic(app.projects().active_id(), |active| match active {
        Some(_) => text("回答将引用当前项目的资料。"),
        None => text("请先在「资料库」选择一个项目。"),
    });
    let error = dynamic(app.agent().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("提问")).children(column().children((
        input,
        row().children((ask, state_line)),
        project_line,
        error,
    )))
}

/// 回答区：回答正文 + 提供方/模型 + 引用列表 + 保存为报告。
fn answer_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.agent().answer(), move |answer| match answer {
        Some(answer) => {
            let header = text(format!("{}（{}）", answer.provider_label, answer.model));
            let refs = if answer.references.is_empty() {
                text("本次回答没有引用资料。").into_any()
            } else {
                let rows = answer
                    .references
                    .iter()
                    .map(|reference| {
                        text(format!(
                            "· {}（{}，来源：{}，分数 {:.2}）\n  {}",
                            reference.title,
                            reference.entity_type,
                            reference.source,
                            reference.score,
                            reference.snippet
                        ))
                    })
                    .collect::<Vec<_>>();
                column().children(rows).into_any()
            };
            column()
                .children((header, text(answer.answer.clone()), refs))
                .into_any()
        }
        None => widget(EmptyState::new("还没有回答"))
            .message("提问后，回答会带编号引用显示在这里。")
            .into_any(),
    });
    let save = nana_ui::runtime::view::button("保存为报告")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::AgentSaveReport));
    let save_job = nana_ui::runtime::view::button("保存为任务")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::AgentSaveJob));
    let save_note = dynamic(app.agent().save_message(), |message| {
        message.as_ref().map(|message| text(message.clone()))
    });
    widget(Card::new().title("回答"))
        .children(column().children((body, row().children((save, save_job, save_note)))))
}
