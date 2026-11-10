//! 索引管线：切片/向量统计、构建嵌入入口、回退说明。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, widget, IntoView};
use nana_ui::runtime::{Activate, Card, EmptyState};

use crate::state::{AppMessage, AppState, AppStateStoreFields, IndexingStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((text("索引管线"), status_card(app.clone()), build_card(app)))
}

/// 索引状态：切片/向量/缺失/过期 + 模型 + 阶段徽标。
fn status_card(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.indexing().status(), move |status| match status {
        Some(status) => {
            let counts = text(format!(
                "切片 {} · 向量 {} · 缺失 {} · 过期 {}",
                status.chunk_count,
                status.embedding_count,
                status.missing_embedding_count,
                status.stale_embedding_count
            ));
            let model = if status.model.is_empty() {
                text("嵌入模型：未配置")
            } else {
                text(format!("嵌入模型：{}", status.model))
            };
            let message = if status.message.is_empty() {
                None
            } else {
                Some(text(status.message.clone()))
            };
            column().children((counts, model, message)).into_any()
        }
        None => widget(EmptyState::new("尚未构建索引"))
            .message("先执行「切片索引」，再构建嵌入以启用语义检索。")
            .into_any(),
    });
    widget(Card::new().title("索引状态")).children(body)
}

/// 构建嵌入：按钮 + 忙碌提示；说明回退条件。
fn build_card(app: Store<AppState>) -> impl IntoView {
    let build = nana_ui::runtime::view::button("构建嵌入索引")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::IndexingBuild));
    let busy = dynamic(app.indexing().building(), |building| {
        if *building {
            text("正在批量生成向量…")
        } else {
            text("")
        }
    });
    let message = dynamic(app.indexing().message(), |message| {
        message.as_ref().map(|message| text(message.clone()))
    });
    widget(Card::new().title("构建")).children(column().children((
        row().children((build, busy)),
        text("嵌入需要已配置的 OpenAI 兼容提供方与密钥；缺失时语义检索自动回退关键词，其余功能不受影响。"),
        message,
    )))
}
