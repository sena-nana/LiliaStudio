use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilter {
    pub project_id: String,
    pub query: String,
    pub entity_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

pub fn search_project(
    connection: &Connection,
    filter: SearchFilter,
) -> rusqlite::Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let query = filter.query.trim();
    if query.is_empty() {
        return Ok(results);
    }
    let include_all = filter.entity_types.is_empty();

    if include_all || filter.entity_types.iter().any(|entity| entity == "entry") {
        search_table(
            connection,
            &mut results,
            "entry",
            "entries",
            "title",
            "summary || ' ' || body || ' ' || tags_json",
            &filter.project_id,
            query,
        )?;
    }
    if include_all || filter.entity_types.iter().any(|entity| entity == "character") {
        search_table(
            connection,
            &mut results,
            "character",
            "characters",
            "name",
            "summary || ' ' || goals || ' ' || motivations || ' ' || faction || ' ' || tags_json",
            &filter.project_id,
            query,
        )?;
    }
    if include_all || filter.entity_types.iter().any(|entity| entity == "event") {
        search_table(
            connection,
            &mut results,
            "event",
            "events",
            "title",
            "description || ' ' || time_label || ' ' || location || ' ' || outcome || ' ' || tags_json",
            &filter.project_id,
            query,
        )?;
    }
    if include_all || filter.entity_types.iter().any(|entity| entity == "axiom") {
        search_table(
            connection,
            &mut results,
            "axiom",
            "axioms",
            "subject",
            "predicate || ' ' || object || ' ' || natural_language || ' ' || tags_json",
            &filter.project_id,
            query,
        )?;
    }

    results.sort_by(|left, right| right.score.total_cmp(&left.score));
    Ok(results)
}

fn search_table(
    connection: &Connection,
    results: &mut Vec<SearchResult>,
    entity_type: &str,
    table: &str,
    title_column: &str,
    body_expr: &str,
    project_id: &str,
    query: &str,
) -> rusqlite::Result<()> {
    let like_query = format!("%{query}%");
    let sql = format!(
        "SELECT id, {title_column}, {body_expr}
         FROM {table}
         WHERE project_id = ?1
           AND deleted_at IS NULL
           AND ({title_column} LIKE ?2 OR {body_expr} LIKE ?2)"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params![project_id, like_query], |row| {
        let title: String = row.get(1)?;
        let body: String = row.get(2)?;
        let score = if title.contains(query) { 2.0 } else { 1.0 };
        Ok(SearchResult {
            entity_type: entity_type.to_string(),
            entity_id: row.get(0)?,
            title,
            snippet: make_snippet(&body, query),
            score,
        })
    })?;
    for row in rows {
        results.push(row?);
    }
    Ok(())
}

fn make_snippet(text: &str, query: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    if let Some(index) = text.find(query) {
        let start = index.saturating_sub(24);
        let end = (index + query.len() + 48).min(text.len());
        text[start..end].to_string()
    } else {
        text.chars().take(72).collect()
    }
}
