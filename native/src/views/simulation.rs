//! 情景模拟：固定情景推演 + 阶段/风险/引用实体报告。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, widget, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{AppMessage, AppState, AppStateStoreFields, SimulationStateStoreFields};

/// Tauri 版固定情景（SimulationView）。
const SCENARIO: &str = "如果北方发生饥荒";
const REFERENCED_ENTITIES: [&str; 2] = ["粮食", "北方城墙"];

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("情景模拟"),
        scenario_card(app.clone()),
        report_card(app),
    ))
}

/// 情景说明 + 运行按钮（对当前项目执行 simulate_scenario）。
fn scenario_card(app: Store<AppState>) -> impl IntoView {
    let run = nana_ui::runtime::view::button("运行模拟")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::SimulationRun));
    let busy = dynamic(app.simulation().running(), |running| {
        if *running {
            text("推演中…")
        } else {
            text("")
        }
    });
    let error = dynamic(app.simulation().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("情景")).children(column().children((
        text(format!("情景：{SCENARIO}")),
        text(format!("关注实体：{}", REFERENCED_ENTITIES.join("、"))),
        row().children((run, busy)),
        error,
    )))
}

/// 报告：阶段、风险、引用实体；报告文本本地可读可复制。
fn report_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.simulation().report(), move |report| match report {
        Some(report) => {
            let phases = if report.phases.is_empty() {
                text("（无阶段）").into_any()
            } else {
                let rows = report
                    .phases
                    .iter()
                    .map(|phase| text(format!("· {}：{}", phase.label, phase.summary)))
                    .collect::<Vec<_>>();
                column().children(rows).into_any()
            };
            let risks = if report.risks.is_empty() {
                text("（无风险提示）").into_any()
            } else {
                let rows = report
                    .risks
                    .iter()
                    .map(|risk| text(format!("· {risk}")))
                    .collect::<Vec<_>>();
                column().children(rows).into_any()
            };
            let entities = if report.referenced_entities.is_empty() {
                text("（无引用实体）").into_any()
            } else {
                text(report.referenced_entities.join("、")).into_any()
            };
            column()
                .children((
                    text(format!("情景：{}", report.scenario)),
                    text("阶段"),
                    phases,
                    text("风险"),
                    risks,
                    text("引用实体"),
                    entities,
                ))
                .into_any()
        }
        None => widget(EmptyState::new("尚未运行模拟"))
            .message("点击「运行模拟」生成推演报告；报告保存在本地，可整段复制。")
            .into_any(),
    });
    widget(Card::new().title("报告")).children(body)
}
