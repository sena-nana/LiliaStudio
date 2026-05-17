use tauri::State;

use crate::{
    db::app_state::AppState,
    services::prompts::{
        copy_prompt_template as copy_prompt_template_record,
        list_prompt_templates as list_prompt_template_records,
        preview_prompt_template as preview_prompt_template_record,
        reset_builtin_prompt_templates as reset_builtin_prompt_template_records,
        save_prompt_template as save_prompt_template_record, PromptTemplate, PromptTemplateDraft,
        PromptTemplatePreview, PromptTemplatePreviewRequest,
    },
};

#[tauri::command]
pub fn list_prompt_templates(state: State<'_, AppState>) -> Result<Vec<PromptTemplate>, String> {
    state.with_database(list_prompt_template_records)
}

#[tauri::command]
pub fn copy_prompt_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<PromptTemplate, String> {
    state.with_database(|connection| copy_prompt_template_record(connection, &template_id))
}

#[tauri::command]
pub fn save_prompt_template(
    state: State<'_, AppState>,
    draft: PromptTemplateDraft,
) -> Result<PromptTemplate, String> {
    state.with_database(|connection| save_prompt_template_record(connection, draft))
}

#[tauri::command]
pub fn reset_builtin_prompt_templates(
    state: State<'_, AppState>,
) -> Result<Vec<PromptTemplate>, String> {
    state.with_database(reset_builtin_prompt_template_records)
}

#[tauri::command]
pub fn preview_prompt_template(request: PromptTemplatePreviewRequest) -> PromptTemplatePreview {
    preview_prompt_template_record(request)
}
