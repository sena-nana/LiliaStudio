use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CharacterTraitState {
    pub values: BTreeMap<String, f32>,
    pub sources: Vec<TraitDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TraitDelta {
    pub source_event_id: String,
    pub trait_name: String,
    pub delta: f32,
    pub reason: String,
}

pub fn apply_trait_delta(state: &mut CharacterTraitState, delta: TraitDelta) {
    let value = state.values.entry(delta.trait_name.clone()).or_insert(0.0);
    *value = (*value + delta.delta).clamp(-1.0, 1.0);
    state.sources.push(delta);
}
