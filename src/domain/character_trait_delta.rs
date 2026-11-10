use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterTraitDeltaRecord {
    pub id: String,
    pub project_id: String,
    pub character_id: String,
    pub source_event_id: String,
    pub trait_name: String,
    pub delta: f32,
    pub reason: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterTraitDeltaRecordDraft {
    pub project_id: String,
    pub character_id: String,
    pub source_event_id: String,
    pub trait_name: String,
    pub delta: f32,
    pub reason: String,
}

pub struct CharacterTraitDeltaRepository<'a> {
    connection: &'a Connection,
}

impl<'a> CharacterTraitDeltaRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(
        &self,
        draft: CharacterTraitDeltaRecordDraft,
    ) -> rusqlite::Result<CharacterTraitDeltaRecord> {
        let id = new_id("trait_delta");
        let timestamp = now();
        self.connection.execute(
            "INSERT INTO character_trait_deltas
             (id, project_id, character_id, source_event_id, trait_name, delta, reason, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                id,
                draft.project_id,
                draft.character_id,
                draft.source_event_id,
                draft.trait_name,
                draft.delta,
                draft.reason,
                timestamp
            ],
        )?;
        self.get(&id)
            .map(|record| record.expect("created trait delta record should exist"))
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<CharacterTraitDeltaRecord>> {
        self.connection
            .query_row(
                "SELECT id, project_id, character_id, source_event_id, trait_name, delta, reason,
                        created_at, updated_at, deleted_at
                 FROM character_trait_deltas
                 WHERE id = ?1",
                params![id],
                map_character_trait_delta_record,
            )
            .optional()
    }

    pub fn list_project(
        &self,
        project_id: &str,
    ) -> rusqlite::Result<Vec<CharacterTraitDeltaRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, character_id, source_event_id, trait_name, delta, reason,
                    created_at, updated_at, deleted_at
             FROM character_trait_deltas
             WHERE project_id = ?1 AND deleted_at IS NULL
             ORDER BY created_at ASC, id ASC",
        )?;
        let records = statement
            .query_map(params![project_id], map_character_trait_delta_record)?
            .collect();
        records
    }

    pub fn soft_delete(&self, id: &str) -> rusqlite::Result<()> {
        let timestamp = now();
        self.connection.execute(
            "UPDATE character_trait_deltas
             SET deleted_at = ?2, updated_at = ?2
             WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(())
    }
}

fn map_character_trait_delta_record(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CharacterTraitDeltaRecord> {
    Ok(CharacterTraitDeltaRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        character_id: row.get(2)?,
        source_event_id: row.get(3)?,
        trait_name: row.get(4)?,
        delta: row.get(5)?,
        reason: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        deleted_at: row.get(9)?,
    })
}
