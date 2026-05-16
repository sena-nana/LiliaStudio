use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSummary {
    pub app_version: String,
    pub platform: String,
    pub database: String,
}

#[tauri::command]
pub fn diagnostics_summary() -> DiagnosticsSummary {
    DiagnosticsSummary {
        app_version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
        database: "sqlite".into(),
    }
}
