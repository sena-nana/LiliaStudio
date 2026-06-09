use tauri::State;

use crate::{
    db::app_state::AppState,
    services::jobs::{
        cancel_job, create_queued_job, current_running_job, list_job_logs, list_jobs, retry_job,
        AiJob, AiJobDraft, AiJobLog,
    },
};

#[tauri::command]
pub fn job_create(state: State<'_, AppState>, draft: AiJobDraft) -> Result<AiJob, String> {
    state.with_database(|connection| create_queued_job(connection, draft))
}

#[tauri::command]
pub fn job_list(state: State<'_, AppState>) -> Result<Vec<AiJob>, String> {
    state.with_database(list_jobs)
}

#[tauri::command]
pub fn job_current(state: State<'_, AppState>) -> Result<Option<AiJob>, String> {
    state.with_database(current_running_job)
}

#[tauri::command]
pub fn job_logs(state: State<'_, AppState>, job_id: String) -> Result<Vec<AiJobLog>, String> {
    state.with_database(|connection| list_job_logs(connection, &job_id))
}

#[tauri::command]
pub fn job_cancel(state: State<'_, AppState>, job_id: String) -> Result<AiJob, String> {
    state.with_database(|connection| cancel_job(connection, &job_id))
}

#[tauri::command]
pub fn job_retry(state: State<'_, AppState>, job_id: String) -> Result<AiJob, String> {
    state.with_database(|connection| retry_job(connection, &job_id))
}
