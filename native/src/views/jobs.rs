//! 任务队列：当前任务 + 任务列表 + 日志查看。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, widget, AnyView, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{AppMessage, AppState, AppStateStoreFields, JobsStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("任务队列"),
        current_card(app.clone()),
        list_card(app.clone()),
        logs_card(app),
    ))
}

fn action_button(
    label: &'static str,
    message: AppMessage,
) -> nana_ui::runtime::view::El<nana_ui::runtime::Button> {
    nana_ui::runtime::view::button(label)
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(message.clone()))
}

/// 当前运行中的任务：状态 + 取消入口。
fn current_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.jobs().current(), move |current| match current {
        Some(job) => column()
            .children((
                text(format!("{} · {}", job.job_type, job.provider_kind)),
                text(format!("状态：{}", job.status)),
                text(job.input_summary.clone()),
                action_button("请求取消", AppMessage::JobCancel(job.id.clone())),
            ))
            .into_any(),
        None => text("当前没有运行中的任务。").into_any(),
    });
    widget(Card::new().title("当前任务")).children(body)
}

fn job_row(job: &ameya_core::services::jobs::AiJob) -> AnyView {
    let info = text(format!(
        "· [{}] {} · {}（创建于 {}）",
        job.status, job.job_type, job.input_summary, job.created_at
    ));
    let logs = action_button("日志", AppMessage::JobOpenLogs(job.id.clone()));
    let retry = action_button("重试", AppMessage::JobRetry(job.id.clone()));
    row().children((info, logs, retry)).into_any()
}

/// 任务列表：逐条展示状态与操作（日志/重试）。
fn list_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.jobs().list(), move |jobs| {
        if jobs.is_empty() {
            widget(EmptyState::new("任务队列为空"))
                .message("智能体回答保存为任务后，会在这里跟踪状态。")
                .into_any()
        } else {
            let rows = jobs.iter().map(job_row).collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    widget(Card::new().title("任务列表")).children(body)
}

/// 日志：只读文本展示最近一次读取的日志。
fn logs_card(app: Store<AppState>) -> impl IntoView {
    let logs = dynamic(app.jobs().open_logs(), move |logs| match logs {
        Some(logs) => text(logs.clone()).into_any(),
        None => text("在任务列表点击「日志」查看执行记录。").into_any(),
    });
    let error = dynamic(app.jobs().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("日志")).children(column().children((logs, error)))
}
