use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::shared::{new_id, now};

pub const JOB_STATUS_QUEUED: &str = "queued";
pub const JOB_STATUS_RUNNING: &str = "running";
pub const JOB_STATUS_SUCCEEDED: &str = "succeeded";
pub const JOB_STATUS_FAILED: &str = "failed";
pub const JOB_STATUS_CANCELLED: &str = "cancelled";

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
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub retry_of_job_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiJobLog {
    pub id: String,
    pub job_id: String,
    pub level: String,
    pub message: String,
    pub created_at: String,
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
    create_job_record(connection, draft, JOB_STATUS_QUEUED, None)
}

pub fn mark_job_running(connection: &Connection, job_id: &str) -> rusqlite::Result<AiJob> {
    let timestamp = now();
    let affected = connection.execute(
        "UPDATE ai_jobs
         SET status = ?2, started_at = COALESCE(started_at, ?3), updated_at = ?3
         WHERE id = ?1",
        params![job_id, JOB_STATUS_RUNNING, timestamp],
    )?;
    ensure_job_was_updated(affected)?;
    append_job_log(connection, job_id, "info", "任务开始执行")?;
    require_job(connection, job_id)
}

pub fn complete_job_success(
    connection: &Connection,
    job_id: &str,
    output_text: String,
) -> rusqlite::Result<AiJob> {
    let timestamp = now();
    let affected = connection.execute(
        "UPDATE ai_jobs
         SET status = ?2, output_text = ?3, error_message = NULL, finished_at = ?4, updated_at = ?4
         WHERE id = ?1",
        params![job_id, JOB_STATUS_SUCCEEDED, output_text, timestamp],
    )?;
    ensure_job_was_updated(affected)?;
    append_job_log(connection, job_id, "info", "任务执行成功")?;
    require_job(connection, job_id)
}

pub fn fail_job(
    connection: &Connection,
    job_id: &str,
    error_message: String,
) -> rusqlite::Result<AiJob> {
    let timestamp = now();
    let sanitized = sanitize_job_message(&error_message);
    let affected = connection.execute(
        "UPDATE ai_jobs
         SET status = ?2, error_message = ?3, finished_at = ?4, updated_at = ?4
         WHERE id = ?1",
        params![job_id, JOB_STATUS_FAILED, sanitized, timestamp],
    )?;
    ensure_job_was_updated(affected)?;
    append_job_log(connection, job_id, "error", &sanitized)?;
    require_job(connection, job_id)
}

pub fn cancel_job(connection: &Connection, job_id: &str) -> rusqlite::Result<AiJob> {
    let timestamp = now();
    let affected = connection.execute(
        "UPDATE ai_jobs
         SET status = ?2, cancel_requested_at = COALESCE(cancel_requested_at, ?3),
             finished_at = COALESCE(finished_at, ?3), updated_at = ?3
         WHERE id = ?1",
        params![job_id, JOB_STATUS_CANCELLED, timestamp],
    )?;
    ensure_job_was_updated(affected)?;
    append_job_log(connection, job_id, "info", "任务已取消")?;
    require_job(connection, job_id)
}

pub fn retry_job(connection: &Connection, job_id: &str) -> rusqlite::Result<AiJob> {
    let original = require_job(connection, job_id)?;
    let retried = create_job_record(
        connection,
        AiJobDraft {
            project_id: original.project_id.clone(),
            provider_kind: original.provider_kind.clone(),
            job_type: original.job_type.clone(),
            input_summary: original.input_summary.clone(),
        },
        JOB_STATUS_QUEUED,
        Some(original.id.clone()),
    )?;
    append_job_log(
        connection,
        &retried.id,
        "info",
        &format!("已从 {} 重试", original.id),
    )?;
    Ok(retried)
}

pub fn append_job_log(
    connection: &Connection,
    job_id: &str,
    level: &str,
    message: &str,
) -> rusqlite::Result<AiJobLog> {
    let id = new_id("joblog");
    let timestamp = now();
    let sanitized = sanitize_job_message(message);
    connection.execute(
        "INSERT INTO ai_job_logs (id, job_id, level, message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, job_id, level, sanitized, timestamp],
    )?;
    require_job_log(connection, &id)
}

pub fn list_job_logs(connection: &Connection, job_id: &str) -> rusqlite::Result<Vec<AiJobLog>> {
    let mut statement = connection.prepare(
        "SELECT id, job_id, level, message, created_at
         FROM ai_job_logs
         WHERE job_id = ?1
         ORDER BY created_at ASC",
    )?;
    let logs = statement.query_map(params![job_id], map_job_log)?.collect();
    logs
}

pub fn current_running_job(connection: &Connection) -> rusqlite::Result<Option<AiJob>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, provider_kind, job_type, status, input_summary, output_text,
                error_message, started_at, finished_at, cancel_requested_at, retry_of_job_id,
                created_at, updated_at
         FROM ai_jobs
         WHERE status = ?1
         ORDER BY started_at DESC, updated_at DESC
         LIMIT 1",
    )?;
    let mut rows = statement.query(params![JOB_STATUS_RUNNING])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_job(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_jobs(connection: &Connection) -> rusqlite::Result<Vec<AiJob>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, provider_kind, job_type, status, input_summary, output_text,
                error_message, started_at, finished_at, cancel_requested_at, retry_of_job_id,
                created_at, updated_at
         FROM ai_jobs
         ORDER BY updated_at DESC",
    )?;
    let jobs = statement.query_map([], map_job)?.collect();
    jobs
}

fn create_job_record(
    connection: &Connection,
    draft: AiJobDraft,
    status: &str,
    retry_of_job_id: Option<String>,
) -> rusqlite::Result<AiJob> {
    let id = new_id("job");
    let timestamp = now();
    connection.execute(
        "INSERT INTO ai_jobs
         (id, project_id, provider_kind, job_type, status, input_summary, output_text, error_message,
          started_at, finished_at, cancel_requested_at, retry_of_job_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', NULL, NULL, NULL, NULL, ?7, ?8, ?8)",
        params![
            id,
            draft.project_id,
            draft.provider_kind,
            draft.job_type,
            status,
            draft.input_summary,
            retry_of_job_id,
            timestamp
        ],
    )?;
    append_job_log(connection, &id, "info", "任务已排队")?;
    require_job(connection, &id)
}

fn ensure_job_was_updated(affected: usize) -> rusqlite::Result<()> {
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

fn require_job(connection: &Connection, id: &str) -> rusqlite::Result<AiJob> {
    get_job(connection, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

fn require_job_log(connection: &Connection, id: &str) -> rusqlite::Result<AiJobLog> {
    get_job_log(connection, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

fn get_job(connection: &Connection, id: &str) -> rusqlite::Result<Option<AiJob>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, provider_kind, job_type, status, input_summary, output_text,
                error_message, started_at, finished_at, cancel_requested_at, retry_of_job_id,
                created_at, updated_at
         FROM ai_jobs WHERE id = ?1",
    )?;
    let mut rows = statement.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_job(row)?))
    } else {
        Ok(None)
    }
}

fn get_job_log(connection: &Connection, id: &str) -> rusqlite::Result<Option<AiJobLog>> {
    let mut statement = connection.prepare(
        "SELECT id, job_id, level, message, created_at
         FROM ai_job_logs WHERE id = ?1",
    )?;
    let mut rows = statement.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_job_log(row)?))
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
        started_at: row.get(8)?,
        finished_at: row.get(9)?,
        cancel_requested_at: row.get(10)?,
        retry_of_job_id: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn map_job_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiJobLog> {
    Ok(AiJobLog {
        id: row.get(0)?,
        job_id: row.get(1)?,
        level: row.get(2)?,
        message: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn sanitize_job_message(message: &str) -> String {
    let mut redacted = message.to_string();
    redacted = redact_assignment_value(&redacted, "api_key=");
    redacted = redact_assignment_value(&redacted, "apiKey=");
    redacted = redact_bearer_tokens(&redacted);
    redact_sk_tokens(&redacted)
}

fn redact_assignment_value(message: &str, needle: &str) -> String {
    let mut output = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(position) = rest.find(needle) {
        let split_at = position + needle.len();
        output.push_str(&rest[..split_at]);
        let tail = &rest[split_at..];
        let value_end = tail
            .find(|ch: char| {
                ch.is_whitespace() || ch == '"' || ch == '\'' || ch == ',' || ch == ';'
            })
            .unwrap_or(tail.len());
        output.push_str("[redacted]");
        rest = &tail[value_end..];
    }
    output.push_str(rest);
    output
}

fn redact_bearer_tokens(message: &str) -> String {
    let mut output = String::new();
    for line in message.split_inclusive('\n') {
        if let Some(position) = line.to_ascii_lowercase().find("authorization: bearer ") {
            let (prefix, tail) = line.split_at(position);
            output.push_str(prefix);
            output.push_str("Authorization: Bearer [redacted]");
            let suffix = &tail["Authorization: Bearer ".len()..];
            let value_end = suffix
                .find(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';')
                .unwrap_or(suffix.len());
            output.push_str(&suffix[value_end..]);
        } else {
            output.push_str(line);
        }
    }
    output
}

fn redact_sk_tokens(message: &str) -> String {
    let mut output = String::new();
    let mut rest = message;
    while let Some(position) = rest.find("sk-") {
        output.push_str(&rest[..position]);
        let tail = &rest[position + 3..];
        let value_end = tail
            .find(|ch: char| {
                ch.is_whitespace() || ch == '"' || ch == '\'' || ch == ',' || ch == ';'
            })
            .unwrap_or(tail.len());
        output.push_str("sk-[redacted]");
        rest = &tail[value_end..];
    }
    output.push_str(rest);
    output
}
