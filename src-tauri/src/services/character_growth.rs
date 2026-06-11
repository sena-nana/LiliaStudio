use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::character_trait_delta::{
    CharacterTraitDeltaRecord, CharacterTraitDeltaRecordDraft, CharacterTraitDeltaRepository,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterGrowthWorkspaceSnapshot {
    pub project_id: String,
    pub records: Vec<CharacterTraitDeltaRecord>,
    pub states: BTreeMap<String, CharacterTraitState>,
}

pub fn apply_trait_delta(state: &mut CharacterTraitState, delta: TraitDelta) {
    let value = state.values.entry(delta.trait_name.clone()).or_insert(0.0);
    *value = (*value + delta.delta).clamp(-1.0, 1.0);
    state.sources.push(delta);
}

pub fn create_trait_delta_record(
    connection: &rusqlite::Connection,
    draft: CharacterTraitDeltaRecordDraft,
) -> rusqlite::Result<CharacterTraitDeltaRecord> {
    CharacterTraitDeltaRepository::new(connection).create(draft)
}

pub fn list_growth_workspace(
    connection: &rusqlite::Connection,
    project_id: &str,
) -> rusqlite::Result<CharacterGrowthWorkspaceSnapshot> {
    let records = CharacterTraitDeltaRepository::new(connection).list_project(project_id)?;

    Ok(CharacterGrowthWorkspaceSnapshot {
        project_id: project_id.to_string(),
        states: build_trait_states_by_character(&records),
        records,
    })
}

fn record_to_trait_delta(record: &CharacterTraitDeltaRecord) -> TraitDelta {
    TraitDelta {
        source_event_id: record.source_event_id.clone(),
        trait_name: record.trait_name.clone(),
        delta: record.delta,
        reason: record.reason.clone(),
    }
}

fn build_trait_states_by_character(
    records: &[CharacterTraitDeltaRecord],
) -> BTreeMap<String, CharacterTraitState> {
    let mut states = BTreeMap::new();
    for record in records {
        let state = states
            .entry(record.character_id.clone())
            .or_insert_with(CharacterTraitState::default);
        apply_trait_delta(state, record_to_trait_delta(record));
    }
    states
}
