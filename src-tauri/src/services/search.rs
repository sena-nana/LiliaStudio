use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::{entry::EntryRepository, entry_rich_text::entry_searchable_text};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MatchRange {
    start: usize,
    end: usize,
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
        search_entries(connection, &mut results, &filter.project_id, query)?;
    }
    if include_all
        || filter
            .entity_types
            .iter()
            .any(|entity| entity == "character")
    {
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

fn search_entries(
    connection: &Connection,
    results: &mut Vec<SearchResult>,
    project_id: &str,
    query: &str,
) -> rusqlite::Result<()> {
    for entry in EntryRepository::new(connection).list_active(project_id)? {
        let body = entry_searchable_text("", &entry.summary, &entry.body, &entry.tags);
        let title_matches = find_case_insensitive(&entry.title, query).is_some();
        let body_matches = find_case_insensitive(&body, query).is_some();
        if title_matches || body_matches {
            results.push(SearchResult {
                entity_type: "entry".to_string(),
                entity_id: entry.id,
                title: entry.title.clone(),
                snippet: make_snippet(&body, query),
                score: if title_matches { 2.0 } else { 1.0 },
            });
        }
    }
    Ok(())
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
        let score = if find_case_insensitive(&title, query).is_some() {
            2.0
        } else {
            1.0
        };
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
    if let Some(range) = find_case_insensitive(text, query) {
        let chars = text.chars().collect::<Vec<_>>();
        let start = range.start.saturating_sub(24);
        let end = (range.end + 48).min(chars.len());
        chars[start..end].iter().collect()
    } else {
        text.chars().take(72).collect()
    }
}

fn find_case_insensitive(text: &str, query: &str) -> Option<MatchRange> {
    if text.is_empty() || query.is_empty() {
        return None;
    }

    let folded_query = query.to_lowercase();
    let mut folded_text = String::new();
    let mut folded_to_original = Vec::new();
    for (original_index, character) in text.chars().enumerate() {
        for folded_character in character.to_lowercase() {
            folded_text.push(folded_character);
            folded_to_original.push(original_index);
        }
    }

    let byte_index = folded_text.find(&folded_query)?;
    let folded_start = folded_text[..byte_index].chars().count();
    let folded_len = folded_query.chars().count();
    let folded_end = folded_start + folded_len;
    let start = *folded_to_original.get(folded_start)?;
    let end = folded_to_original
        .get(folded_end.saturating_sub(1))
        .map(|index| index + 1)
        .unwrap_or_else(|| text.chars().count());

    Some(MatchRange { start, end })
}

#[cfg(test)]
mod tests {
    use super::{find_case_insensitive, make_snippet, MatchRange};

    #[test]
    fn snippet_handles_utf8_boundaries() {
        let snippet = make_snippet("潮汐规则 月光阔剑只能在涨潮时启动", "涨潮");

        assert!(snippet.contains("涨潮"));
    }

    #[test]
    fn snippet_uses_case_insensitive_match_range() {
        let snippet = make_snippet("A secret Moon Gate opens at dusk", "moon");

        assert!(snippet.contains("Moon Gate"));
    }

    #[test]
    fn case_insensitive_match_maps_back_to_original_text_range() {
        assert_eq!(
            find_case_insensitive("A secret Moon Gate", "moon"),
            Some(MatchRange { start: 9, end: 13 })
        );
    }
}
