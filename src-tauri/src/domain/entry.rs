use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{decode_json, encode_json, new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub project_id: String,
    pub entry_type: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntryDraft {
    pub project_id: String,
    pub entry_type: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
}

pub struct EntryRepository<'a> {
    connection: &'a Connection,
}

impl<'a> EntryRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, draft: EntryDraft) -> rusqlite::Result<Entry> {
        let id = new_id("entry");
        let timestamp = now();
        let tags = encode_json(&draft.tags)?;
        self.connection.execute(
            "INSERT INTO entries
             (id, project_id, entry_type, title, summary, body, tags_json, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                id,
                draft.project_id,
                draft.entry_type,
                draft.title,
                draft.summary,
                draft.body,
                tags,
                draft.status,
                timestamp
            ],
        )?;
        self.get(&id).map(|entry| entry.expect("created entry should exist"))
    }

    pub fn update(&self, id: &str, draft: EntryDraft) -> rusqlite::Result<Entry> {
        let tags = encode_json(&draft.tags)?;
        self.connection.execute(
            "UPDATE entries
             SET entry_type = ?2, title = ?3, summary = ?4, body = ?5, tags_json = ?6,
                 status = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                id,
                draft.entry_type,
                draft.title,
                draft.summary,
                draft.body,
                tags,
                draft.status,
                now()
            ],
        )?;
        self.get(id).map(|entry| entry.expect("updated entry should exist"))
    }

    pub fn soft_delete(&self, id: &str) -> rusqlite::Result<()> {
        let timestamp = now();
        self.connection.execute(
            "UPDATE entries SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(())
    }

    pub fn restore(&self, id: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE entries SET deleted_at = NULL, updated_at = ?2 WHERE id = ?1",
            params![id, now()],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Entry>> {
        self.connection
            .query_row(
                "SELECT id, project_id, entry_type, title, summary, body, tags_json, status,
                        created_at, updated_at, deleted_at
                 FROM entries WHERE id = ?1",
                params![id],
                map_entry,
            )
            .optional()
    }

    pub fn list_active(&self, project_id: &str) -> rusqlite::Result<Vec<Entry>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, entry_type, title, summary, body, tags_json, status,
                    created_at, updated_at, deleted_at
             FROM entries
             WHERE project_id = ?1 AND deleted_at IS NULL
             ORDER BY updated_at DESC",
        )?;
        let entries = statement.query_map(params![project_id], map_entry)?.collect();
        entries
    }
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        project_id: row.get(1)?,
        entry_type: row.get(2)?,
        title: row.get(3)?,
        summary: row.get(4)?,
        body: row.get(5)?,
        tags: decode_json(row.get(6)?, 6)?,
        status: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
    })
}
