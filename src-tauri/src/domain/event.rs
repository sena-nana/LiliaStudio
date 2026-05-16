use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{decode_json, encode_json, new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub time_label: String,
    pub sort_key: i64,
    pub start_label: String,
    pub end_label: String,
    pub location: String,
    pub importance: i64,
    pub outcome: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventDraft {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub time_label: String,
    pub sort_key: i64,
    pub start_label: String,
    pub end_label: String,
    pub location: String,
    pub importance: i64,
    pub outcome: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventParticipant {
    pub id: String,
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventParticipantDraft {
    pub entity_type: String,
    pub entity_id: String,
    pub role: String,
}

pub struct EventRepository<'a> {
    connection: &'a Connection,
}

impl<'a> EventRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(
        &self,
        draft: EventDraft,
        participants: Vec<EventParticipantDraft>,
    ) -> rusqlite::Result<Event> {
        let id = new_id("event");
        let timestamp = now();
        let tags = encode_json(&draft.tags)?;
        self.connection.execute(
            "INSERT INTO events
             (id, project_id, title, description, time_label, sort_key, start_label, end_label,
              location, importance, outcome, tags_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
            params![
                id,
                draft.project_id,
                draft.title,
                draft.description,
                draft.time_label,
                draft.sort_key,
                draft.start_label,
                draft.end_label,
                draft.location,
                draft.importance,
                draft.outcome,
                tags,
                timestamp
            ],
        )?;

        self.replace_participants(&id, participants)?;
        self.get(&id).map(|event| event.expect("created event should exist"))
    }

    pub fn replace_participants(
        &self,
        event_id: &str,
        participants: Vec<EventParticipantDraft>,
    ) -> rusqlite::Result<()> {
        self.connection
            .execute("DELETE FROM event_participants WHERE event_id = ?1", params![event_id])?;
        for participant in participants {
            self.connection.execute(
                "INSERT INTO event_participants
                 (id, event_id, entity_type, entity_id, role, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    new_id("participant"),
                    event_id,
                    participant.entity_type,
                    participant.entity_id,
                    participant.role,
                    now()
                ],
            )?;
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Event>> {
        self.connection
            .query_row(
                "SELECT id, project_id, title, description, time_label, sort_key, start_label,
                        end_label, location, importance, outcome, tags_json, created_at,
                        updated_at, deleted_at
                 FROM events WHERE id = ?1",
                params![id],
                map_event,
            )
            .optional()
    }

    pub fn list_active(&self, project_id: &str) -> rusqlite::Result<Vec<Event>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, description, time_label, sort_key, start_label,
                    end_label, location, importance, outcome, tags_json, created_at,
                    updated_at, deleted_at
             FROM events
             WHERE project_id = ?1 AND deleted_at IS NULL
             ORDER BY sort_key ASC, updated_at DESC",
        )?;
        let events = statement.query_map(params![project_id], map_event)?.collect();
        events
    }

    pub fn soft_delete(&self, id: &str) -> rusqlite::Result<()> {
        let timestamp = now();
        self.connection.execute(
            "UPDATE events SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(())
    }

    pub fn list_participants(&self, event_id: &str) -> rusqlite::Result<Vec<EventParticipant>> {
        let mut statement = self.connection.prepare(
            "SELECT id, event_id, entity_type, entity_id, role, created_at
             FROM event_participants
             WHERE event_id = ?1
             ORDER BY created_at ASC",
        )?;
        let participants = statement
            .query_map(params![event_id], |row| {
                Ok(EventParticipant {
                    id: row.get(0)?,
                    event_id: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect();
        participants
    }
}

fn map_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<Event> {
    Ok(Event {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        time_label: row.get(4)?,
        sort_key: row.get(5)?,
        start_label: row.get(6)?,
        end_label: row.get(7)?,
        location: row.get(8)?,
        importance: row.get(9)?,
        outcome: row.get(10)?,
        tags: decode_json(row.get(11)?, 11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        deleted_at: row.get(14)?,
    })
}
