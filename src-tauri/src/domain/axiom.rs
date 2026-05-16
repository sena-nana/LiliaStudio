use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{decode_json, encode_json, new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Axiom {
    pub id: String,
    pub project_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub scope_time: String,
    pub scope_location: String,
    pub certainty: f64,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub natural_language: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AxiomDraft {
    pub project_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub scope_time: String,
    pub scope_location: String,
    pub certainty: f64,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub natural_language: String,
    pub tags: Vec<String>,
}

pub struct AxiomRepository<'a> {
    connection: &'a Connection,
}

impl<'a> AxiomRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, draft: AxiomDraft) -> rusqlite::Result<Axiom> {
        let id = new_id("axiom");
        let timestamp = now();
        let tags = encode_json(&draft.tags)?;
        self.connection.execute(
            "INSERT INTO axioms
             (id, project_id, subject, predicate, object, scope_time, scope_location, certainty,
              source_entity_type, source_entity_id, natural_language, tags_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
            params![
                id,
                draft.project_id,
                draft.subject,
                draft.predicate,
                draft.object,
                draft.scope_time,
                draft.scope_location,
                draft.certainty,
                draft.source_entity_type,
                draft.source_entity_id,
                draft.natural_language,
                tags,
                timestamp
            ],
        )?;
        self.get(&id).map(|axiom| axiom.expect("created axiom should exist"))
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Axiom>> {
        self.connection
            .query_row(
                "SELECT id, project_id, subject, predicate, object, scope_time, scope_location,
                        certainty, source_entity_type, source_entity_id, natural_language,
                        tags_json, created_at, updated_at, deleted_at
                 FROM axioms WHERE id = ?1",
                params![id],
                map_axiom,
            )
            .optional()
    }

    pub fn search(&self, project_id: &str, query: &str) -> rusqlite::Result<Vec<Axiom>> {
        let like_query = format!("%{query}%");
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, subject, predicate, object, scope_time, scope_location,
                    certainty, source_entity_type, source_entity_id, natural_language,
                    tags_json, created_at, updated_at, deleted_at
             FROM axioms
             WHERE project_id = ?1
               AND deleted_at IS NULL
               AND (subject LIKE ?2 OR predicate LIKE ?2 OR object LIKE ?2 OR natural_language LIKE ?2)
             ORDER BY updated_at DESC",
        )?;
        let axioms = statement
            .query_map(params![project_id, like_query], map_axiom)?
            .collect();
        axioms
    }
}

fn map_axiom(row: &rusqlite::Row<'_>) -> rusqlite::Result<Axiom> {
    Ok(Axiom {
        id: row.get(0)?,
        project_id: row.get(1)?,
        subject: row.get(2)?,
        predicate: row.get(3)?,
        object: row.get(4)?,
        scope_time: row.get(5)?,
        scope_location: row.get(6)?,
        certainty: row.get(7)?,
        source_entity_type: row.get(8)?,
        source_entity_id: row.get(9)?,
        natural_language: row.get(10)?,
        tags: decode_json(row.get(11)?, 11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        deleted_at: row.get(14)?,
    })
}
