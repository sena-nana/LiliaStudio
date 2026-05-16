use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HealthInfo {
    pub app_version: String,
    pub platform: String,
    pub app_data_dir: String,
}

#[tauri::command]
pub fn health_check(app: tauri::AppHandle) -> Result<HealthInfo, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data directory: {error}"))?;

    Ok(HealthInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::HealthInfo;

    #[test]
    fn serializes_health_info_as_frontend_contract() {
        let health = HealthInfo {
            app_version: "0.1.0".to_string(),
            platform: "windows".to_string(),
            app_data_dir: "C:/Users/demo/AppData/Roaming/ameya".to_string(),
        };

        let json = serde_json::to_value(health).expect("health info should serialize");

        assert_eq!(json["appVersion"], "0.1.0");
        assert_eq!(json["platform"], "windows");
        assert_eq!(json["appDataDir"], "C:/Users/demo/AppData/Roaming/ameya");
    }
}
