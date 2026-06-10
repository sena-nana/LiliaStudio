use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        axiom::{Axiom, AxiomRepository},
        character::{Character, CharacterRepository},
        entry::{Entry, EntryRepository},
        entry_rich_text::entry_rich_text_to_plain_text,
        event::{Event, EventRepository},
        shared::{encode_json, new_id, now},
    },
    vector::{
        chunking::{chunk_text, TextChunk},
        search::cosine_similarity,
    },
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
    pub estimated_tokens: usize,
    pub updated_at: String,
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

#[derive(Debug, Clone, PartialEq)]
pub struct VectorCandidate {
    pub chunk_id: String,
    pub source_type: String,
    pub source_id: String,
    pub text: String,
    pub vector: Vec<f32>,
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
    delete_inactive_source_chunks(connection, project_id)?;

    for entry in EntryRepository::new(connection).list_active(project_id)? {
        index_source_document(
            connection,
            project_id,
            "entry",
            &entry.id,
            &entry_document_text(&entry),
            max_chars,
        )?;
    }
    for character in CharacterRepository::new(connection).list_active(project_id)? {
        index_source_document(
            connection,
            project_id,
            "character",
            &character.id,
            &character_document_text(&character),
            max_chars,
        )?;
    }
    for event in EventRepository::new(connection).list_active(project_id)? {
        index_source_document(
            connection,
            project_id,
            "event",
            &event.id,
            &event_document_text(&event),
            max_chars,
        )?;
    }
    for axiom in AxiomRepository::new(connection).list_active(project_id)? {
        index_source_document(
            connection,
            project_id,
            "axiom",
            &axiom.id,
            &axiom_document_text(&axiom),
            max_chars,
        )?;
    }

    list_project_chunks(connection, project_id)
}

fn delete_inactive_source_chunks(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<()> {
    for (source_type, table) in [
        ("entry", "entries"),
        ("character", "characters"),
        ("event", "events"),
        ("axiom", "axioms"),
    ] {
        let sql = format!(
            "DELETE FROM document_chunks
             WHERE project_id = ?1
               AND source_type = ?2
               AND NOT EXISTS (
                 SELECT 1 FROM {table}
                 WHERE {table}.id = document_chunks.source_id
                   AND {table}.project_id = ?1
                   AND {table}.deleted_at IS NULL
               )"
        );
        connection.execute(&sql, params![project_id, source_type])?;
    }
    Ok(())
}

fn index_source_document(
    connection: &Connection,
    project_id: &str,
    source_type: &str,
    source_id: &str,
    text: &str,
    max_chars: usize,
) -> rusqlite::Result<()> {
    let chunks = chunk_text(text, max_chars);
    for chunk in &chunks {
        upsert_chunk(connection, project_id, source_type, source_id, chunk)?;
    }
    delete_extra_chunks(connection, project_id, source_type, source_id, chunks.len())
}

fn upsert_chunk(
    connection: &Connection,
    project_id: &str,
    source_type: &str,
    source_id: &str,
    chunk: &TextChunk,
) -> rusqlite::Result<()> {
    let id = new_id("chunk");
    let timestamp = now();
    connection.execute(
        "INSERT INTO document_chunks
         (id, project_id, source_type, source_id, ordinal, text, content_hash,
          estimated_tokens, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
         ON CONFLICT(source_type, source_id, ordinal)
         DO UPDATE SET
           text = excluded.text,
           content_hash = excluded.content_hash,
           estimated_tokens = excluded.estimated_tokens,
           updated_at = excluded.updated_at
         WHERE document_chunks.text <> excluded.text
            OR document_chunks.content_hash <> excluded.content_hash
            OR document_chunks.estimated_tokens <> excluded.estimated_tokens",
        params![
            id,
            project_id,
            source_type,
            source_id,
            chunk.ordinal as i64,
            chunk.text,
            chunk.content_hash,
            chunk.estimated_tokens as i64,
            timestamp
        ],
    )?;
    Ok(())
}

fn delete_extra_chunks(
    connection: &Connection,
    project_id: &str,
    source_type: &str,
    source_id: &str,
    chunk_count: usize,
) -> rusqlite::Result<()> {
    connection.execute(
        "DELETE FROM document_chunks
         WHERE project_id = ?1 AND source_type = ?2 AND source_id = ?3 AND ordinal >= ?4",
        params![project_id, source_type, source_id, chunk_count as i64],
    )?;
    Ok(())
}

fn list_project_chunks(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<Vec<DocumentChunkRecord>> {
    let mut statement = connection.prepare(
        "SELECT id, project_id, source_type, source_id, ordinal, text, content_hash,
                estimated_tokens, updated_at
         FROM document_chunks
         WHERE project_id = ?1
         ORDER BY source_type, source_id, ordinal",
    )?;
    let rows = statement.query_map(params![project_id], map_chunk)?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

fn entry_document_text(entry: &Entry) -> String {
    let mut fields = Vec::new();
    push_text_field(&mut fields, "标题", &entry.title);
    push_text_field(
        &mut fields,
        "摘要",
        &entry_rich_text_to_plain_text(&entry.summary),
    );
    push_text_field(
        &mut fields,
        "正文",
        &entry_rich_text_to_plain_text(&entry.body),
    );
    push_list_field(&mut fields, "标签", &entry.tags);
    fields.join("\n")
}

fn character_document_text(character: &Character) -> String {
    let mut fields = Vec::new();
    push_text_field(&mut fields, "姓名", &character.name);
    push_list_field(&mut fields, "别名", &character.aliases);
    push_text_field(&mut fields, "简介", &character.summary);
    push_text_field(&mut fields, "外貌", &character.appearance);
    push_text_field(&mut fields, "目标", &character.goals);
    push_text_field(&mut fields, "动机", &character.motivations);
    push_text_field(&mut fields, "恐惧", &character.fears);
    push_text_field(&mut fields, "阵营", &character.faction);
    push_list_field(&mut fields, "标签", &character.tags);
    fields.join("\n")
}

fn event_document_text(event: &Event) -> String {
    let mut fields = Vec::new();
    push_text_field(&mut fields, "标题", &event.title);
    push_text_field(&mut fields, "描述", &event.description);
    push_text_field(&mut fields, "时间", &event.time_label);
    push_text_field(&mut fields, "开始", &event.start_label);
    push_text_field(&mut fields, "结束", &event.end_label);
    push_text_field(&mut fields, "地点", &event.location);
    push_text_field(&mut fields, "重要性", &event.importance.to_string());
    push_text_field(&mut fields, "结果", &event.outcome);
    push_list_field(&mut fields, "标签", &event.tags);
    fields.join("\n")
}

fn axiom_document_text(axiom: &Axiom) -> String {
    let mut fields = Vec::new();
    push_text_field(&mut fields, "主语", &axiom.subject);
    push_text_field(&mut fields, "谓词", &axiom.predicate);
    push_text_field(&mut fields, "宾语", &axiom.object);
    push_text_field(&mut fields, "时间范围", &axiom.scope_time);
    push_text_field(&mut fields, "地点范围", &axiom.scope_location);
    push_text_field(&mut fields, "可信度", &axiom.certainty.to_string());
    push_optional_field(&mut fields, "来源类型", axiom.source_entity_type.as_deref());
    push_optional_field(&mut fields, "来源 ID", axiom.source_entity_id.as_deref());
    push_text_field(&mut fields, "自然语言", &axiom.natural_language);
    push_list_field(&mut fields, "标签", &axiom.tags);
    fields.join("\n")
}

fn push_text_field(fields: &mut Vec<String>, label: &str, value: &str) {
    let trimmed = value.trim();
    if !trimmed.is_empty() {
        fields.push(format!("{label}: {trimmed}"));
    }
}

fn push_optional_field(fields: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_text_field(fields, label, value);
    }
}

fn push_list_field(fields: &mut Vec<String>, label: &str, values: &[String]) {
    let text = values
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    push_text_field(fields, label, &text);
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
    let candidates = load_vector_candidates(connection, project_id)?;
    Ok(rank_vector_candidates(candidates, query_vector, limit))
}

pub fn load_vector_candidates(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<Vec<VectorCandidate>> {
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
        Ok(VectorCandidate {
            chunk_id: row.get(0)?,
            source_type: row.get(1)?,
            source_id: row.get(2)?,
            text: row.get(3)?,
            vector,
        })
    })?;

    let mut candidates = Vec::new();
    for row in rows {
        candidates.push(row?);
    }
    Ok(candidates)
}

pub fn rank_vector_candidates(
    candidates: Vec<VectorCandidate>,
    query_vector: Vec<f32>,
    limit: usize,
) -> Vec<VectorMatch> {
    let mut matches: Vec<_> = candidates
        .into_iter()
        .map(|candidate| VectorMatch {
            chunk_id: candidate.chunk_id,
            source_type: candidate.source_type,
            source_id: candidate.source_id,
            text: candidate.text,
            score: cosine_similarity(&query_vector, &candidate.vector).unwrap_or(0.0),
        })
        .collect();
    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    matches.truncate(limit);
    matches
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
        estimated_tokens: row.get::<_, i64>(7)? as usize,
        updated_at: row.get(8)?,
    })
}
