//! 规则审计：运行样例审计 → 冲突报告 + 修复建议（与 Tauri 版样例一致）。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, widget, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{AppMessage, AppState, AppStateStoreFields, AuditStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("规则审计"),
        run_card(app.clone()),
        conflicts_card(app.clone()),
        repairs_card(app),
    ))
}

/// 运行样例审计：固定两条「月光金属」状态事实（与 Tauri 版一致）。
fn run_card(app: Store<AppState>) -> impl IntoView {
    let run = nana_ui::runtime::view::button("运行样例审计")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::LoadAudit));
    let error = dynamic(app.audit().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("样例审计")).children(column().children((
        text("样例事实：月光金属在第三纪·北方同时为固态与液态（互斥公理冲突）。"),
        row().children((run, error)),
    )))
}

fn conflict_type_label(value: &str) -> &str {
    match value {
        "mutually_exclusive_axioms" => "互斥公理",
        _ => value,
    }
}

/// 冲突列表：每条含类型与说明。
fn conflicts_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.audit().conflicts(), |conflicts| {
        if conflicts.is_empty() {
            widget(EmptyState::new("没有检测到冲突"))
                .message("点击「运行样例审计」检查公理一致性。")
                .into_any()
        } else {
            let rows = conflicts
                .iter()
                .map(|conflict| {
                    text(format!(
                        "· [{}] {}（涉及事实：{}）",
                        conflict_type_label(&conflict.conflict_type),
                        conflict.message,
                        conflict.fact_ids.join("、")
                    ))
                })
                .collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    widget(Card::new().title("冲突报告")).children(body)
}

/// 修复建议：跟随第一条冲突生成。
fn repairs_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.audit().repairs(), |repairs| {
        if repairs.is_empty() {
            None
        } else {
            let rows = repairs
                .iter()
                .map(|repair| {
                    text(format!(
                        "· {}：{}（影响：{}）",
                        repair.title, repair.description, repair.impact
                    ))
                })
                .collect::<Vec<_>>();
            Some(column().children(rows))
        }
    });
    widget(Card::new().title("修复建议")).children(body)
}
