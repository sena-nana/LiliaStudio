use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        entry::EntryRepository,
        shared::{encode_json, new_id, now},
    },
    vector::{chunking::chunk_text, search::cosine_similarity},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunkRecord {
    pub id: String,
    pub project_id: String,
    pub source_type: String,
    pub source_id: String,
    pub ordinal: usize,
    pub text: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorMatch {
    pub chunk_id: String,
    pub source_type: String,
    pub source_id: String,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPack {
    pub project_id: String,
    pub query: String,
    pub items: Vec<ContextItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextItem {
    pub source_type: String,
    pub source_id: String,
    pub text: String,
    pub score: f32,
}

pub fn index_project_chunks(
    connection: &Connection,
    project_id: &str,
    max_chars: usize,
) -> rusqlite::Result<Vec<DocumentChunkRecord>> {
    let entries = EntryRepository::new(connection).list_active(project_id)?;
    let mut records = Vec::new();

    for entry in entries {
        let text = format!("{}\n{}\n{}", entry.title, entry.summary, entry.body);
        for chunk in chunk_text(&text, max_chars) {
            let id = new_id("chunk");
            let timestamp = now();
            connection.execute(
                "INSERT INTO document_chunks
                 (id, project_id, source_type, source_id, ordinal, text, content_hash, created_at, updated_at)
                 VALUES (?1, ?2, 'entry', ?3, ?4, ?5, ?6, ?7, ?7)
                 ON CONFLICT(source_type, source_id, ordinal)
                 DO UPDATE SET text = excluded.text, content_hash = excluded.content_hash, updated_at = excluded.updated_at",
                params![
                    id,
                    project_id,
                    entry.id,
                    chunk.ordinal as i64,
                    chunk.text,
                    chunk.content_hash,
                    timestamp
                ],
            )?;
        }
    }

    let mut statement = connection.prepare(
        "SELECT id, project_id, source_type, source_id, ordinal, text, content_hash
         FROM document_chunks
         WHERE project_id = ?1
         ORDER BY source_type, source_id, ordinal",
    )?;
    let rows = statement.query_map(params![project_id], map_chunk)?;
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

pub fn upsert_embedding(
    connection: &Connection,
    chunk_id: &str,
    model: &str,
    vector: Vec<f32>,
) -> rusqlite::Result<()> {
    let id = new_id("embedding");
    let timestamp = now();
    let vector_json = encode_json(&vector)?;
    connection.execute(
        "INSERT INTO embeddings (id, chunk_id, model, dimensions, vector_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
         ON CONFLICT(chunk_id, model)
         DO UPDATE SET dimensions = excluded.dimensions, vector_json = excluded.vector_json, updated_at = excluded.updated_at",
        params![id, chunk_id, model, vector.len() as i64, vector_json, timestamp],
    )?;
    Ok(())
}

pub fn vector_search(
    connection: &Connection,
    project_id: &str,
    query_vector: Vec<f32>,
    limit: usize,
) -> rusqlite::Result<Vec<VectorMatch>> {
    let mut statement = connection.prepare(
        "SELECT c.id, c.source_type, c.source_id, c.text, e.vector_json
         FROM document_chunks c
         JOIN embeddings e ON e.chunk_id = c.id
         WHERE c.project_id = ?1",
    )?;
    let rows = statement.query_map(params![project_id], |row| {
        let vector_json: String = row.get(4)?;
        let vector: Vec<f32> = serde_json::from_str(&vector_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let score = cosine_similarity(&query_vector, &vector).unwrap_or(0.0);
        Ok(VectorMatch {
            chunk_id: row.get(0)?,
            source_type: row.get(1)?,
            source_id: row.get(2)?,
            text: row.get(3)?,
            score,
        })
    })?;

    let mut matches = Vec::new();
    for row in rows {
        matches.push(row?);
    }
    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    matches.truncate(limit);
    Ok(matches)
}

pub fn build_context_pack(
    connection: &Connection,
    project_id: &str,
    query: &str,
    query_vector: Vec<f32>,
) -> rusqlite::Result<ContextPack> {
    let items = vector_search(connection, project_id, query_vector, 8)?
        .into_iter()
        .map(|item| ContextItem {
            source_type: item.source_type,
            source_id: item.source_id,
            text: item.text,
            score: item.score,
        })
        .collect();
    Ok(ContextPack {
        project_id: project_id.to_string(),
        query: query.to_string(),
        items,
    })
}

fn map_chunk(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentChunkRecord> {
    Ok(DocumentChunkRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        source_type: row.get(2)?,
        source_id: row.get(3)?,
        ordinal: row.get::<_, i64>(4)? as usize,
        text: row.get(5)?,
        content_hash: row.get(6)?,
    })
}
