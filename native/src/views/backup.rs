//! 备份与迁移：导出当前项目归档 / 粘贴归档导入。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, text_area, widget, IntoView};
use nana_ui::runtime::{Activate, Card};

use crate::state::{AppMessage, AppState, AppStateStoreFields, BackupStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("备份与迁移"),
        export_card(app.clone()),
        import_card(app),
    ))
}

/// 导出：当前项目 → 归档 JSON 文本（可复制保存）。
fn export_card(app: Store<AppState>) -> impl IntoView {
    let export = nana_ui::runtime::view::button("导出当前项目")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::BackupExport));
    let busy = dynamic(app.backup().busy(), |busy| {
        if *busy {
            text("正在导出…")
        } else {
            text("")
        }
    });
    let output = dynamic(app.backup().export_text(), |export_text| {
        if export_text.is_empty() {
            None
        } else {
            Some(text(export_text.clone()))
        }
    });
    widget(Card::new().title("导出")).children(column().children((
        row().children((export, busy)),
        text("导出的归档文本包含项目的资料、角色、事件、规则与关系，可保存为 .json 文件。"),
        output,
    )))
}

/// 导入：粘贴归档文本 → 导入为新项目。
fn import_card(app: Store<AppState>) -> impl IntoView {
    let backup = app.backup();
    let input = text_area()
        .placeholder("粘贴此前导出的归档 JSON 文本")
        .model(backup.import_text());
    let import = nana_ui::runtime::view::button("导入归档")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::BackupImport));
    let note = dynamic(app.backup().notice(), |notice| {
        notice.as_ref().map(|notice| text(notice.clone()))
    });
    let error = dynamic(app.backup().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("导入")).children(column().children((
        input,
        row().children((import, note)),
        error,
    )))
}
