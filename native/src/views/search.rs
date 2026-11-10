//! 搜索工作台：检索条件 + 关键词结果 + 语义结果（带状态与回退提示）。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, text_input, widget, IntoView};
use nana_ui::runtime::Stack;
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{AppMessage, AppState, AppStateStoreFields, SearchStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("搜索"),
        query_card(app.clone()),
        keyword_card(app.clone()),
        semantic_card(app),
    ))
}

/// 检索条件：查询词 + 检索按钮；空查询给出提示。
fn query_card(app: Store<AppState>) -> impl IntoView {
    let search = app.search();
    let input = text_input()
        .placeholder("输入关键词，检索项目内资料/角色/事件/规则/关系")
        .model(search.query());
    let submit = nana_ui::runtime::view::button("检索")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::SearchSubmit));
    let busy = dynamic(app.search().loading(), |loading| {
        if *loading {
            text("检索中…")
        } else {
            text("")
        }
    });
    widget(Card::new().title("检索条件"))
        .children(column().children((widget(Stack::fill_row(8.0)).children((input, submit)), busy)))
}

fn result_row(
    result: &ameya_core::services::search::SearchResult,
) -> nana_ui::runtime::view::El<nana_ui::runtime::Text> {
    text(format!(
        "· {}（{}，分数 {:.2}）\n  {}",
        result.title, result.entity_type, result.score, result.snippet
    ))
}

/// 关键词结果（search_entities）。
fn keyword_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.search().keyword_results(), |results| {
        if results.is_empty() {
            widget(EmptyState::new("暂无关键词结果"))
                .message("输入查询词后点击「检索」。")
                .into_any()
        } else {
            let rows = results.iter().map(result_row).collect::<Vec<_>>();
            column().children(rows).into_any()
        }
    });
    widget(Card::new().title("关键词结果")).children(body)
}

/// 语义结果（search_semantic）：状态徽标 + 结果或回退原因。
fn semantic_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.search().semantic(), move |semantic| match semantic {
        Some(response) => {
            let (label, note) = match response.status {
                ameya_core::services::search::SemanticSearchStatus::Ready => {
                    ("就绪", format!("模型：{}", response.model))
                }
                ameya_core::services::search::SemanticSearchStatus::Degraded => {
                    ("已回退", response.message.clone())
                }
            };
            let header = row().children((nana_ui::runtime::view::status_badge(label), text(note)));
            if response.items.is_empty() {
                column()
                    .children((header, text("语义通道没有命中结果")))
                    .into_any()
            } else {
                let rows = response.items.iter().map(result_row).collect::<Vec<_>>();
                column()
                    .children((header, column().children(rows)))
                    .into_any()
            }
        }
        None => widget(EmptyState::new("尚未执行语义检索"))
            .message("语义检索依赖向量索引；未配置嵌入提供方时自动回退关键词。")
            .into_any(),
    });
    widget(Card::new().title("语义结果")).children(body)
}
