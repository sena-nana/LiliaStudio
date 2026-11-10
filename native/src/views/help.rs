//! 帮助：命令面板用法 + 校验命令清单。

use nana_ui::runtime::view::IntoView;
use nana_ui::runtime::view::{column, text, widget};
use nana_ui::runtime::Card;

pub fn page() -> impl IntoView {
    column().children((text("帮助"), palette_card(), verify_card()))
}

/// 命令面板：Ctrl+K 打开，Esc 关闭；覆盖 14 条常用命令。
fn palette_card() -> impl IntoView {
    widget(Card::new().title("命令面板")).children(column().children((
        text("Ctrl+K 打开命令面板，Esc 关闭；列表覆盖新建项目、搜索与全部页面跳转。"),
        text("命令按页面分组；键盘 ↑↓ 选择，回车执行。"),
    )))
}

/// 工程校验命令（与仓库 README 一致）。
fn verify_card() -> impl IntoView {
    widget(Card::new().title("校验命令")).children(column().children((
        text("cargo test --workspace"),
        text("cargo check -p ameya-nana --features nana-runtime"),
        text("cmake -S native -B build/native-validation && ctest --test-dir build/native-validation -C Debug"),
    )))
}
