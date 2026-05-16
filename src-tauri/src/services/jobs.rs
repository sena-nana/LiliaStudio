use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::shared::{new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiJob {
    pub id: String,
    pub project_id: Option<String>,
    pub provider_kind: String,
    pub job_type: String,
    pub status: String,
    pub input_summary: String,
    pub output_text: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiJobDraft {
    pub project_id: Option<String>,
    pub provider_kind: String,
    pub job_type: String,
    pub input_summary: String,
}

pub fn create_queued_job(connection: &Connection, draft: AiJobDraft) -> rusqlite::Result<AiJob> {
    let id = new_id("job");
    let timestamp = now();
    connection.execute(
        "INSERT INTO ai_jobs
         (id, project_id, provider_kind, job_type, status, input_summary, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'queued', ?5, ?6, ?6)",
        params![
            id,
            draft.project_id,
            draft.provider_kind,
            draft.job_type,
            draft.input_summary,
            timestamp
        ],
    )?;
    get_job(connection, &id).map(|job| job.expect("created job should exist"))
}

pub fn list_jobs(connection: &Connection) -> rusqlite::Result<Vec<AiJob>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, provider_kind, job_type, status, input_summary, output_text,
                error_message, created_at, updated_at
         FROM ai_jobs
         ORDER BY updated_at DESC",
    )?;
    let jobs = statement.query_map([], map_job)?.collect();
    jobs
}

fn get_job(connection: &Connection, id: &str) -> rusqlite::Result<Option<AiJob>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, provider_kind, job_type, status, input_summary, output_text,
                error_message, created_at, updated_at
         FROM ai_jobs WHERE id = ?1",
    )?;
    let mut rows = statement.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_job(row)?))
    } else {
        Ok(None)
    }
}

fn map_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiJob> {
    Ok(AiJob {
        id: row.get(0)?,
        project_id: row.get(1)?,
        provider_kind: row.get(2)?,
        job_type: row.get(3)?,
        status: row.get(4)?,
        input_summary: row.get(5)?,
        output_text: row.get(6)?,
        error_message: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}
