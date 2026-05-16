pub mod commands;
pub mod db;

#[cfg(test)]
pub mod test_support;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::health::health_check])
        .run(tauri::generate_context!())
        .expect("failed to run Ameya");
}