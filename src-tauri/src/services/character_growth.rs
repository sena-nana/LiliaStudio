use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::{
    character::CharacterRepository,
    character_trait_delta::{
        CharacterTraitDeltaRecord, CharacterTraitDeltaRecordDraft, CharacterTraitDeltaRepository,
    },
    event::EventRepository,
    project::ProjectRepository,
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
    ensure_active_growth_links(connection, &draft)?;
    CharacterTraitDeltaRepository::new(connection).create(draft)
}

pub fn list_growth_workspace(
    connection: &rusqlite::Connection,
    project_id: &str,
) -> rusqlite::Result<CharacterGrowthWorkspaceSnapshot> {
    let records = list_active_project_records(connection, project_id)?;

    Ok(CharacterGrowthWorkspaceSnapshot {
        states: build_trait_states_by_character(&records),
        records,
    })
}

fn ensure_active_growth_links(
    connection: &rusqlite::Connection,
    draft: &CharacterTraitDeltaRecordDraft,
) -> rusqlite::Result<()> {
    let project = ProjectRepository::new(connection)
        .get(&draft.project_id)?
        .ok_or(rusqlite::Error::InvalidQuery)?;
    CharacterRepository::new(connection)
        .get(&draft.character_id)?
        .filter(|character| character.project_id == project.id && character.deleted_at.is_none())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    EventRepository::new(connection)
        .get(&draft.source_event_id)?
        .filter(|event| event.project_id == project.id && event.deleted_at.is_none())
        .ok_or(rusqlite::Error::InvalidQuery)?;

    Ok(())
}

fn list_active_project_records(
    connection: &rusqlite::Connection,
    project_id: &str,
) -> rusqlite::Result<Vec<CharacterTraitDeltaRecord>> {
    let records = CharacterTraitDeltaRepository::new(connection).list_project(project_id)?;
    let characters = CharacterRepository::new(connection).list_active(project_id)?;
    let events = EventRepository::new(connection).list_active(project_id)?;
    let active_character_ids: std::collections::BTreeSet<_> = characters
        .into_iter()
        .map(|character| character.id)
        .collect();
    let active_event_ids: std::collections::BTreeSet<_> =
        events.into_iter().map(|event| event.id).collect();

    Ok(records
        .into_iter()
        .filter(|record| {
            active_character_ids.contains(&record.character_id)
                && active_event_ids.contains(&record.source_event_id)
        })
        .collect())
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
