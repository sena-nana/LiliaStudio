use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{decode_json, encode_json, new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub appearance: String,
    pub goals: String,
    pub motivations: String,
    pub fears: String,
    pub faction: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDraft {
    pub project_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub appearance: String,
    pub goals: String,
    pub motivations: String,
    pub fears: String,
    pub faction: String,
    pub tags: Vec<String>,
}

pub struct CharacterRepository<'a> {
    connection: &'a Connection,
}

impl<'a> CharacterRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, draft: CharacterDraft) -> rusqlite::Result<Character> {
        let id = new_id("character");
        let timestamp = now();
        let aliases = encode_json(&draft.aliases)?;
        let tags = encode_json(&draft.tags)?;
        self.connection.execute(
            "INSERT INTO characters
             (id, project_id, name, aliases_json, summary, appearance, goals, motivations,
              fears, faction, tags_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
            params![
                id,
                draft.project_id,
                draft.name,
                aliases,
                draft.summary,
                draft.appearance,
                draft.goals,
                draft.motivations,
                draft.fears,
                draft.faction,
                tags,
                timestamp
            ],
        )?;
        self.get(&id)
            .map(|character| character.expect("created character should exist"))
    }

    pub fn soft_delete(&self, id: &str) -> rusqlite::Result<()> {
        let timestamp = now();
        self.connection.execute(
            "UPDATE characters SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Character>> {
        self.connection
            .query_row(
                "SELECT id, project_id, name, aliases_json, summary, appearance, goals,
                        motivations, fears, faction, tags_json, created_at, updated_at, deleted_at
                 FROM characters WHERE id = ?1",
                params![id],
                map_character,
            )
            .optional()
    }

    pub fn list_active(&self, project_id: &str) -> rusqlite::Result<Vec<Character>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, name, aliases_json, summary, appearance, goals,
                    motivations, fears, faction, tags_json, created_at, updated_at, deleted_at
             FROM characters
             WHERE project_id = ?1 AND deleted_at IS NULL
             ORDER BY updated_at DESC",
        )?;
        let characters = statement.query_map(params![project_id], map_character)?.collect();
        characters
    }

    pub fn restore(&self, id: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE characters SET deleted_at = NULL, updated_at = ?2 WHERE id = ?1",
            params![id, now()],
        )?;
        Ok(())
    }
}

fn map_character(row: &rusqlite::Row<'_>) -> rusqlite::Result<Character> {
    Ok(Character {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        aliases: decode_json(row.get(3)?, 3)?,
        summary: row.get(4)?,
        appearance: row.get(5)?,
        goals: row.get(6)?,
        motivations: row.get(7)?,
        fears: row.get(8)?,
        faction: row.get(9)?,
        tags: decode_json(row.get(10)?, 10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        deleted_at: row.get(13)?,
    })
}
