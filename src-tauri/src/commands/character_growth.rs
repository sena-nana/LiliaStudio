use tauri::State;

use crate::{
    db::app_state::AppState,
    domain::character_trait_delta::{CharacterTraitDeltaRecord, CharacterTraitDeltaRecordDraft},
    services::character_growth::{
        apply_trait_delta, create_trait_delta_record, list_growth_workspace,
        CharacterGrowthWorkspaceSnapshot, CharacterTraitState, TraitDelta,
    },
};

#[tauri::command]
pub fn character_growth_preview_trait_delta(
    mut state: CharacterTraitState,
    delta: TraitDelta,
) -> CharacterTraitState {
    apply_trait_delta(&mut state, delta);
    state
}

#[tauri::command]
pub fn character_growth_workspace(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<CharacterGrowthWorkspaceSnapshot, String> {
    state.with_database(|connection| list_growth_workspace(connection, &project_id))
}

#[tauri::command]
pub fn character_growth_create_record(
    state: State<'_, AppState>,
    draft: CharacterTraitDeltaRecordDraft,
) -> Result<CharacterTraitDeltaRecord, String> {
    state.with_database(|connection| create_trait_delta_record(connection, draft))
}
