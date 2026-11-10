//! 事件演化时间线（§1.6）：events 按 sortKey 升序；行 = 时间 + 标题 + 描述。

use ameya_core::domain::event::Event;
use nana_ui::runtime::view::Store;
use nana_ui::runtime::view::{
    column, computed, dynamic, empty_state, list_item, text, widget, IntoView, StorePath,
};
use nana_ui::runtime::{Card, List};

use crate::state::{
    AppState, AppStateStoreFields, LibraryStateStoreFields, ProjectsStateStoreFields,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    let projects = app.projects();
    let library = app.library();
    let state = {
        let active_id = projects.active_id();
        let events = library.events();
        computed(move || (active_id.get(), events.get()))
    };
    let body = dynamic(
        state,
        move |(active_id, events): &(Option<String>, Vec<Event>)| {
            if active_id.is_none() {
                return empty_state("未选择项目")
                    .message("先选择一个项目，再查看事件时间线。")
                    .into_any();
            }
            let mut sorted: Vec<&Event> = events.iter().collect();
            sorted.sort_by_key(|event| event.sort_key);
            if sorted.is_empty() {
                return empty_state("暂无事件")
                    .message("在对象工作台的事件集合里创建事件后，这里按时间顺序排列。")
                    .into_any();
            }
            let rows = sorted
                .iter()
                .map(|event| {
                    list_item(event.title.clone())
                        .note(if event.time_label.is_empty() {
                            "未定时间".to_string()
                        } else {
                            event.time_label.clone()
                        })
                        .detail(if !event.description.is_empty() {
                            event.description.clone()
                        } else if !event.outcome.is_empty() {
                            event.outcome.clone()
                        } else {
                            "暂无描述".to_string()
                        })
                })
                .collect::<Vec<_>>();
            widget(List::new().label("事件时间线"))
                .children(rows)
                .into_any()
        },
    );
    column().gap(12.0).children((
        text("事件时间线"),
        widget(Card::new().title("事件流")).children(body),
    ))
}
