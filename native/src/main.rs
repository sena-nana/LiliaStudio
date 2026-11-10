//! 桌面入口。`nana-runtime` 特性下启动 hosted Ameya 应用；无特性时保留
//! 无头 CLI（`--snapshot` / `--render-pages`），供 C++ 几何校验器消费。

#[cfg(feature = "nana-runtime")]
fn main() -> Result<(), nana_ui::HostedRunError> {
    use ameya_nana::app::AmeyaApp;
    use nana_ui::{
        shared_store, ApplicationIdentity, DiagnosticsConfig, FileStore, NanaApplication,
        RuntimeApplication, WindowDescriptor,
    };

    // 离线快照导出：走真实 headless 布局（GPU 离屏），替代旧 ViewBuilder
    // 几何，供 C++ 校验器与 PNG 渲染脚本消费。
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--render-pages" {
            let directory = args
                .next()
                .unwrap_or_else(|| "artifacts/offline-pages".to_string());
            ameya_nana::snapshot_export::export_real_layout_pages(std::path::Path::new(&directory))
                .map_err(|error| {
                    eprintln!("failed to render offline pages: {error}");
                    std::process::exit(1);
                })?;
            println!(
                "rendered {} routes × 2 modes to {directory}",
                ameya_nana::Route::ALL.len()
            );
            return Ok(());
        }
        eprintln!("usage: ameya-nana [--render-pages <directory>]");
        std::process::exit(2);
    }

    let builder = NanaApplication::builder(ApplicationIdentity::new(
        "dev.ameya.studio",
        "Ameya",
        env!("CARGO_PKG_VERSION"),
    ))
    .diagnostics(DiagnosticsConfig::default());
    let descriptor = WindowDescriptor::new("Ameya")
        .initial_size(1280.0, 800.0)
        .minimum_size(960.0, 640.0)
        .persist_key("main");
    // 主题 / 侧栏折叠状态持久化在应用数据目录（local-storage.bin）。
    let kv = NanaApplication::paths()
        .map(|paths| paths.data().to_path_buf())
        .or_else(|| nana_ui_platform::app_data_dir("dev.ameya.studio"))
        .and_then(|dir| FileStore::open(dir).ok())
        .map(shared_store);
    match kv {
        Some(store) => builder.run_with_store::<RuntimeApplication<AmeyaApp>>(descriptor, store),
        None => builder.run::<RuntimeApplication<AmeyaApp>>(descriptor),
    }
}

#[cfg(not(feature = "nana-runtime"))]
fn main() {
    use ameya_nana::{NativeApp, Route};
    let app = NativeApp::default();
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--render-pages" {
            let directory = args
                .next()
                .unwrap_or_else(|| "artifacts/offline-pages".to_string());
            let collapsed = args.next().as_deref() == Some("--collapsed");
            let app = NativeApp {
                route: Route::Home,
                sidebar_collapsed: collapsed,
                selected_project_id: None,
                status_message: None,
                project_draft_name: "未命名项目".to_string(),
                project_draft_description: "由 NanaUI 工作区创建".to_string(),
            };
            app.render_offline_pages(&directory)
                .unwrap_or_else(|error| panic!("failed to render offline pages: {error}"));
            println!(
                "rendered {} routes to {} (sidebar={})",
                Route::ALL.len(),
                directory,
                if collapsed { "collapsed" } else { "expanded" }
            );
            return;
        }
        if flag != "--snapshot" {
            eprintln!("usage: ameya-nana [--snapshot|--render-pages <directory>]");
            std::process::exit(2);
        }
    }
    // Deterministic output is used by the headless geometry validator.
    print!("{}", app.snapshot());
}
