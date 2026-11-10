//! 工作台首页：快速开始 + 项目概览 + 环境状态。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, text, text_input, widget, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{
    AppMessage, AppState, AppStateStoreFields, DiagnosticsStateStoreFields,
    ProjectsStateStoreFields,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("工作台"),
        quick_start(app.clone()),
        projects_overview(app.clone()),
        environment(app),
    ))
}

/// 快速开始：新建项目（名称 + 简介）。
fn quick_start(app: Store<AppState>) -> impl IntoView {
    let projects = app.projects();
    let create = column().gap(6.0).children((
        text_input()
            .placeholder("项目名称，默认：未命名宇宙")
            .model(projects.create_name()),
        text_input()
            .placeholder("一句话简介（可选）")
            .model(projects.create_description()),
        button("新建项目", AppMessage::HomeCreateProject),
    ));
    let error = dynamic(app.projects().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("快速开始")).children(column().children((create, error)))
}

/// 项目概览：当前项目 + 项目列表，空态给引导。
fn projects_overview(app: Store<AppState>) -> impl IntoView {
    let overview = dynamic(app.projects().list(), move |projects| {
        if projects.is_empty() {
            widget(EmptyState::new("还没有项目"))
                .message("在上方输入名称创建第一个世界观项目。")
                .into_any()
        } else {
            let rows = projects
                .iter()
                .map(|project| {
                    let description = if project.description.trim().is_empty() {
                        "暂无简介".to_string()
                    } else {
                        project.description.trim().to_string()
                    };
                    text(format!("· {}（{description}）", project.name))
                })
                .collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    widget(Card::new().title("项目概览")).children(overview)
}

/// 环境状态：本地数据目录与 AI 提供方概况。
fn environment(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.diagnostics().database(), |database| {
        let database = database.as_deref().unwrap_or("未就绪");
        text(format!(
            "本地数据库：{database}。AI 提供方在「设置」中配置。"
        ))
    });
    widget(Card::new().title("环境状态")).children(body)
}

fn button(label: &'static str, message: AppMessage) -> impl IntoView {
    nana_ui::runtime::view::button(label)
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(message.clone()))
}
