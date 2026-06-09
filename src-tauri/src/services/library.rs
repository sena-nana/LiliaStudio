use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::{
    axiom::{Axiom, AxiomRepository},
    character::{Character, CharacterRepository},
    entry::{Entry, EntryRepository},
    event::{Event, EventRepository},
    relation::{Relation, RelationRepository},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryProjectSnapshot {
    pub project_id: String,
    pub entries: Vec<Entry>,
    pub characters: Vec<Character>,
    pub events: Vec<Event>,
    pub axioms: Vec<Axiom>,
    pub relations: Vec<Relation>,
}

pub fn load_project_snapshot(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<LibraryProjectSnapshot> {
    Ok(LibraryProjectSnapshot {
        project_id: project_id.to_string(),
        entries: EntryRepository::new(connection).list_active(project_id)?,
        characters: CharacterRepository::new(connection).list_active(project_id)?,
        events: EventRepository::new(connection).list_active(project_id)?,
        axioms: AxiomRepository::new(connection).search(project_id, "")?,
        relations: RelationRepository::new(connection).list_project(project_id)?,
    })
}
