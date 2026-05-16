use crate::services::character_growth::{apply_trait_delta, CharacterTraitState, TraitDelta};

#[tauri::command]
pub fn preview_trait_delta(mut state: CharacterTraitState, delta: TraitDelta) -> CharacterTraitState {
    apply_trait_delta(&mut state, delta);
    state
}
