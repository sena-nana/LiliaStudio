use tauri::State;

use crate::{
    db::app_state::AppState,
    domain::{
        axiom::{Axiom, AxiomDraft, AxiomRepository},
        character::{Character, CharacterDraft, CharacterRepository},
        entry::{Entry, EntryDraft, EntryRepository},
        event::{Event, EventDraft, EventParticipantDraft, EventRepository},
        relation::{EntityRef, Relation, RelationDraft, RelationRepository},
    },
    services::library::{load_project_snapshot, LibraryProjectSnapshot},
};

#[tauri::command]
pub fn library_project_snapshot(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<LibraryProjectSnapshot, String> {
    state.with_database(|connection| load_project_snapshot(connection, &project_id))
}

#[tauri::command]
pub fn library_create_entry(
    state: State<'_, AppState>,
    draft: EntryDraft,
) -> Result<Entry, String> {
    state.with_database(|connection| EntryRepository::new(connection).create(draft))
}

#[tauri::command]
pub fn library_update_entry(
    state: State<'_, AppState>,
    id: String,
    draft: EntryDraft,
) -> Result<Entry, String> {
    state.with_database(|connection| EntryRepository::new(connection).update(&id, draft))
}

#[tauri::command]
pub fn library_delete_entry(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| EntryRepository::new(connection).soft_delete(&id))
}

#[tauri::command]
pub fn library_create_character(
    state: State<'_, AppState>,
    draft: CharacterDraft,
) -> Result<Character, String> {
    state.with_database(|connection| CharacterRepository::new(connection).create(draft))
}

#[tauri::command]
pub fn library_update_character(
    state: State<'_, AppState>,
    id: String,
    draft: CharacterDraft,
) -> Result<Character, String> {
    state.with_database(|connection| CharacterRepository::new(connection).update(&id, draft))
}

#[tauri::command]
pub fn library_delete_character(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| CharacterRepository::new(connection).soft_delete(&id))
}

#[tauri::command]
pub fn library_create_event(
    state: State<'_, AppState>,
    draft: EventDraft,
    participants: Vec<EventParticipantDraft>,
) -> Result<Event, String> {
    state.with_database(|connection| EventRepository::new(connection).create(draft, participants))
}

#[tauri::command]
pub fn library_update_event(
    state: State<'_, AppState>,
    id: String,
    draft: EventDraft,
    participants: Vec<EventParticipantDraft>,
) -> Result<Event, String> {
    state.with_database(|connection| {
        EventRepository::new(connection).update(&id, draft, participants)
    })
}

#[tauri::command]
pub fn library_delete_event(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| EventRepository::new(connection).soft_delete(&id))
}

#[tauri::command]
pub fn library_create_axiom(
    state: State<'_, AppState>,
    draft: AxiomDraft,
) -> Result<Axiom, String> {
    state.with_database(|connection| AxiomRepository::new(connection).create(draft))
}

#[tauri::command]
pub fn library_update_axiom(
    state: State<'_, AppState>,
    id: String,
    draft: AxiomDraft,
) -> Result<Axiom, String> {
    state.with_database(|connection| AxiomRepository::new(connection).update(&id, draft))
}

#[tauri::command]
pub fn library_delete_axiom(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| AxiomRepository::new(connection).soft_delete(&id))
}

#[tauri::command]
pub fn library_list_backlinks(
    state: State<'_, AppState>,
    target: EntityRef,
) -> Result<Vec<Relation>, String> {
    state.with_database(|connection| RelationRepository::new(connection).list_backlinks(&target))
}

#[tauri::command]
pub fn library_create_relation(
    state: State<'_, AppState>,
    draft: RelationDraft,
) -> Result<Relation, String> {
    state.with_database(|connection| RelationRepository::new(connection).create(draft))
}

#[tauri::command]
pub fn library_update_relation(
    state: State<'_, AppState>,
    id: String,
    draft: RelationDraft,
) -> Result<Relation, String> {
    state.with_database(|connection| RelationRepository::new(connection).update(&id, draft))
}

#[tauri::command]
pub fn library_delete_relation(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_database(|connection| RelationRepository::new(connection).soft_delete(&id))
}
