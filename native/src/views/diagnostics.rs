//! 诊断：环境信息（版本/平台/数据库）+ 刷新。

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{column, dynamic, row, text, widget, IntoView, StorePath};
use nana_ui::runtime::{Activate, Card};

use crate::state::{AppMessage, AppState, AppStateStoreFields, DiagnosticsStateStoreFields};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((text("诊断"), environment_card(app)))
}

fn environment_card(app: Store<AppState>) -> impl IntoView {
    let platform = app.diagnostics().platform();
    let database = app.diagnostics().database();
    let body = dynamic(app.diagnostics().app_version(), move |version| {
        let platform = platform.get();
        let database = database.get();
        column().children((
            text(format!(
                "应用版本：{}",
                version.as_deref().unwrap_or("未知")
            )),
            text(format!("平台：{}", platform.as_deref().unwrap_or("未知"))),
            text(format!(
                "数据库：{}",
                database.as_deref().unwrap_or("未就绪")
            )),
        ))
    });
    let refresh = nana_ui::runtime::view::button("刷新")
        .on_cx(|_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::LoadDiagnostics));
    let error = dynamic(app.diagnostics().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    widget(Card::new().title("环境信息"))
        .children(column().children((body, row().children((refresh, error)))))
}
