pub mod ai;
pub mod commands;
pub mod db;
pub mod domain;
pub mod logic;
pub mod services;
pub mod vector;
mod window_state;
pub mod windows;

pub mod test_support;

type WindowsRuntime = tauri_runtime_wry::Wry<tauri::EventLoopMessage>;

const MAIN_WINDOW_LABEL: &str = "main";
const BG: tauri::utils::config::Color = tauri::utils::config::Color(0x18, 0x18, 0x18, 0xFF);

pub fn run() {
    let builder = tauri::Builder::<WindowsRuntime>::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            use tauri::Manager;

            let app_data_dir = app.path().app_data_dir()?;
            let state = db::app_state::AppState::initialize(app_data_dir)?;
            app.manage(state);
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.set_background_color(Some(BG));
                if let Some(state) = window_state::load_main_window_state(app.handle()) {
                    window_state::restore_main_window_state(&window, state);
                }
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::{Manager, WindowEvent};

            if window.label() != MAIN_WINDOW_LABEL {
                return;
            }
            if matches!(
                event,
                WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
            ) {
                if let Some(webview_window) = window.get_webview_window(MAIN_WINDOW_LABEL) {
                    window_state::persist_main_window_state(&window.app_handle(), &webview_window);
                }
            }
        });
    commands::register_handlers(builder)
        .run(tauri::generate_context!())
        .expect("failed to run Ameya");
}
