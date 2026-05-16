use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::shared::{new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: String,
}

impl EntityRef {
    pub fn entry(entity_id: String) -> Self {
        Self {
            entity_type: "entry".into(),
            entity_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub id: String,
    pub project_id: String,
    pub source: EntityRef,
    pub target: EntityRef,
    pub relation_type: String,
    pub description: String,
    pub confidence: f64,
    pub directed: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelationDraft {
    pub project_id: String,
    pub source: EntityRef,
    pub target: EntityRef,
    pub relation_type: String,
    pub description: String,
    pub confidence: f64,
    pub directed: bool,
}

pub struct RelationRepository<'a> {
    connection: &'a Connection,
}

impl<'a> RelationRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, draft: RelationDraft) -> rusqlite::Result<Relation> {
        let id = new_id("relation");
        let timestamp = now();
        self.connection.execute(
            "INSERT INTO relations
             (id, project_id, source_type, source_id, target_type, target_id, relation_type,
              description, confidence, directed, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![
                id,
                draft.project_id,
                draft.source.entity_type,
                draft.source.entity_id,
                draft.target.entity_type,
                draft.target.entity_id,
                draft.relation_type,
                draft.description,
                draft.confidence,
                draft.directed as i64,
                timestamp
            ],
        )?;
        self.get(&id)
            .map(|relation| relation.expect("created relation should exist"))
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Relation>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, source_type, source_id, target_type, target_id, relation_type,
                    description, confidence, directed, created_at, updated_at, deleted_at
             FROM relations WHERE id = ?1",
        )?;
        let mut rows = statement.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_relation(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_backlinks(&self, target: &EntityRef) -> rusqlite::Result<Vec<Relation>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, source_type, source_id, target_type, target_id, relation_type,
                    description, confidence, directed, created_at, updated_at, deleted_at
             FROM relations
             WHERE target_type = ?1 AND target_id = ?2 AND deleted_at IS NULL
             ORDER BY updated_at DESC",
        )?;
        let relations = statement
            .query_map(params![target.entity_type, target.entity_id], map_relation)?
            .collect();
        relations
    }

    pub fn list_project(&self, project_id: &str) -> rusqlite::Result<Vec<Relation>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, source_type, source_id, target_type, target_id, relation_type,
                    description, confidence, directed, created_at, updated_at, deleted_at
             FROM relations
             WHERE project_id = ?1 AND deleted_at IS NULL
             ORDER BY updated_at DESC",
        )?;
        let relations = statement.query_map(params![project_id], map_relation)?.collect();
        relations
    }
}

fn map_relation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Relation> {
    Ok(Relation {
        id: row.get(0)?,
        project_id: row.get(1)?,
        source: EntityRef {
            entity_type: row.get(2)?,
            entity_id: row.get(3)?,
        },
        target: EntityRef {
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
        },
        relation_type: row.get(6)?,
        description: row.get(7)?,
        confidence: row.get(8)?,
        directed: row.get::<_, i64>(9)? != 0,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        deleted_at: row.get(12)?,
    })
}
