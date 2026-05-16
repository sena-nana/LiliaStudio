use tauri::State;

use crate::{
    db::app_state::AppState,
    services::prompts::{list_prompt_templates, PromptTemplate},
};

#[tauri::command]
pub fn list_prompt_templates_command(
    state: State<'_, AppState>,
) -> Result<Vec<PromptTemplate>, String> {
    state.with_database(list_prompt_templates)
}
