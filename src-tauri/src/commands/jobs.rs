use tauri::State;

use crate::{
    db::app_state::AppState,
    services::jobs::{create_queued_job, list_jobs, AiJob, AiJobDraft},
};

#[tauri::command]
pub fn create_ai_job(state: State<'_, AppState>, draft: AiJobDraft) -> Result<AiJob, String> {
    state.with_database(|connection| create_queued_job(connection, draft))
}

#[tauri::command]
pub fn list_ai_jobs(state: State<'_, AppState>) -> Result<Vec<AiJob>, String> {
    state.with_database(list_jobs)
}
