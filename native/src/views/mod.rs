//! 16 条路由的页面视图。`page` 按 store 里的当前路由动态切换；
//! 控制台页面与工作台页面均在本目录各模块内完成布局。

mod agent;
mod audit;
mod backup;
mod diagnostics;
mod graph;
mod growth;
mod help;
mod home;
mod indexing;
mod jobs;
mod projects;
mod prompts;
mod search;
mod settings;
mod simulation;
mod timeline;

use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{dynamic, widget, AnyView, IntoView};
use nana_ui::runtime::{ScrollAxes, ScrollView};
use nana_ui::RuntimeProgramContext;

use crate::state::{AppMessage, AppState, AppStateStoreFields};
use crate::Route;

/// 路由分发：route 变化时重建 primary 分支。
pub fn page(app: Store<AppState>, _context: RuntimeProgramContext<AppMessage>) -> impl IntoView {
    widget(ScrollView::new(ScrollAxes::Vertical)).children(dynamic(app.route(), move |route| {
        let app = app.clone();
        match route {
            Route::Home => home::page(app).into_any(),
            Route::Projects => projects::page(app).into_any(),
            Route::Growth => growth::page(app).into_any(),
            Route::Timeline => timeline::page(app).into_any(),
            Route::Audit => audit::page(app).into_any(),
            Route::Graph => graph::page(app).into_any(),
            Route::Search => search::page(app).into_any(),
            Route::Indexing => indexing::page(app).into_any(),
            Route::Backup => backup::page(app).into_any(),
            Route::Simulation => simulation::page(app).into_any(),
            Route::Agent => agent::page(app).into_any(),
            Route::Settings => settings::page(app).into_any(),
            Route::Help => help::page().into_any(),
            Route::Diagnostics => diagnostics::page(app).into_any(),
            Route::Jobs => jobs::page(app).into_any(),
            Route::PromptTemplates => prompts::page(app).into_any(),
        }
    }))
}

/// 定宽竖直面板：多栏工作台里给每栏显式宽度，避免栏内输入控件把
/// 同行兄弟挤出窗口（NanaUI 布局在 Shrink 行里不收缩子项）。
pub(crate) fn pane(width: f32, view: impl IntoView) -> AnyView {
    widget(nana_ui::runtime::Stack::column(0.0).with_layout(|layout| {
        layout.width = Some(nana_ui::runtime::LengthSpec::Px(width));
    }))
    .children(view)
    .into_any()
}
