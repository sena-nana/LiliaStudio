use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::shared::{new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDraft {
    pub name: String,
    pub description: String,
}

pub struct ProjectRepository<'a> {
    connection: &'a Connection,
}

impl<'a> ProjectRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, draft: ProjectDraft) -> rusqlite::Result<Project> {
        let id = new_id("project");
        let timestamp = now();
        self.connection.execute(
            "INSERT INTO projects (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, draft.name, draft.description, timestamp],
        )?;
        self.get(&id).map(|project| project.expect("created project should exist"))
    }

    pub fn update(&self, id: &str, draft: ProjectDraft) -> rusqlite::Result<Project> {
        self.connection.execute(
            "UPDATE projects
             SET name = ?2, description = ?3, updated_at = ?4
             WHERE id = ?1",
            params![id, draft.name, draft.description, now()],
        )?;
        self.get(id).map(|project| project.expect("updated project should exist"))
    }

    pub fn archive(&self, id: &str) -> rusqlite::Result<()> {
        let timestamp = now();
        self.connection.execute(
            "UPDATE projects SET archived_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<Project>> {
        self.connection
            .query_row(
                "SELECT id, name, description, created_at, updated_at, archived_at
                 FROM projects WHERE id = ?1",
                params![id],
                map_project,
            )
            .optional()
    }

    pub fn list_active(&self) -> rusqlite::Result<Vec<Project>> {
        self.list_with_clause("WHERE archived_at IS NULL")
    }

    pub fn list_all(&self) -> rusqlite::Result<Vec<Project>> {
        self.list_with_clause("")
    }

    pub fn create_with_name_suffix(&self, draft: ProjectDraft, suffix: &str) -> rusqlite::Result<Project> {
        self.create(ProjectDraft {
            name: format!("{}{}", draft.name, suffix),
            description: draft.description,
        })
    }

    fn list_with_clause(&self, clause: &str) -> rusqlite::Result<Vec<Project>> {
        let sql = format!(
            "SELECT id, name, description, created_at, updated_at, archived_at
             FROM projects {clause} ORDER BY updated_at DESC"
        );
        let mut statement = self.connection.prepare(&sql)?;
        let projects = statement.query_map([], map_project)?.collect();
        projects
    }
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        archived_at: row.get(5)?,
    })
}
