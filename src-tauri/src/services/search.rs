use std::collections::HashSet;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        axiom::AxiomRepository, character::CharacterRepository, entry::EntryRepository,
        entry_rich_text::entry_searchable_text, event::EventRepository,
    },
    services::rag::VectorMatch,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SemanticSearchStatus {
    Ready,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchRequest {
    pub project_id: String,
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResponse {
    pub status: SemanticSearchStatus,
    pub message: String,
    pub model: String,
    pub items: Vec<SearchResult>,
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

pub fn semantic_search_results(
    connection: &Connection,
    mut matches: Vec<VectorMatch>,
    limit: usize,
) -> rusqlite::Result<Vec<SearchResult>> {
    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for item in matches {
        if !seen.insert((item.source_type.clone(), item.source_id.clone())) {
            continue;
        }
        results.push(SearchResult {
            title: load_entity_title(connection, &item.source_type, &item.source_id)?
                .unwrap_or_else(|| item.source_id.clone()),
            entity_type: item.source_type,
            entity_id: item.source_id,
            snippet: make_snippet(&item.text, ""),
            score: item.score as f64,
        });
        if results.len() >= limit {
            break;
        }
    }
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
        push_search_result_if_matches(results, "entry", entry.id, entry.title, &body, query);
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
    let sql = format!(
        "SELECT id, {title_column}, {body_expr}
         FROM {table}
         WHERE project_id = ?1
           AND deleted_at IS NULL"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params![project_id], |row| {
        let title: String = row.get(1)?;
        let body: String = row.get(2)?;
        Ok((row.get::<_, String>(0)?, title, body))
    })?;
    for row in rows {
        let (entity_id, title, body) = row?;
        push_search_result_if_matches(results, entity_type, entity_id, title, &body, query);
    }
    Ok(())
}

fn push_search_result_if_matches(
    results: &mut Vec<SearchResult>,
    entity_type: &str,
    entity_id: String,
    title: String,
    body: &str,
    query: &str,
) {
    let title_matches = find_case_insensitive(&title, query).is_some();
    let body_matches = find_case_insensitive(body, query).is_some();
    if title_matches || body_matches {
        results.push(SearchResult {
            entity_type: entity_type.to_string(),
            entity_id,
            title,
            snippet: make_snippet(body, query),
            score: if title_matches { 2.0 } else { 1.0 },
        });
    }
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

fn load_entity_title(
    connection: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> rusqlite::Result<Option<String>> {
    match entity_type {
        "entry" => Ok(EntryRepository::new(connection)
            .get(entity_id)?
            .map(|entry| entry.title)),
        "character" => Ok(CharacterRepository::new(connection)
            .get(entity_id)?
            .map(|character| character.name)),
        "event" => Ok(EventRepository::new(connection)
            .get(entity_id)?
            .map(|event| event.title)),
        "axiom" => Ok(AxiomRepository::new(connection)
            .get(entity_id)?
            .map(|axiom| axiom.subject)),
        _ => Ok(None),
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
    use super::{find_case_insensitive, make_snippet, semantic_search_results, MatchRange};
    use crate::{
        domain::{
            entry::{EntryDraft, EntryRepository},
            project::{ProjectDraft, ProjectRepository},
        },
        services::rag::{index_project_chunks, VectorMatch},
        test_support::migrated_memory_database,
    };

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

    #[test]
    fn semantic_search_deduplicates_entity_matches_by_highest_score() {
        let connection = migrated_memory_database();
        let project = ProjectRepository::new(&connection)
            .create(ProjectDraft {
                name: "语义搜索项目".into(),
                description: String::new(),
            })
            .unwrap();
        let entry = EntryRepository::new(&connection)
            .create(EntryDraft {
                project_id: project.id.clone(),
                entry_type: "item".into(),
                title: "月光阔剑".into(),
                summary: "潮汐能武器".into(),
                body: "由精灵锻造技艺制造。".into(),
                tags: vec![],
                status: "draft".into(),
            })
            .unwrap();
        let chunks = index_project_chunks(&connection, &project.id, 16).unwrap();

        let results = semantic_search_results(
            &connection,
            vec![
                VectorMatch {
                    chunk_id: chunks[0].id.clone(),
                    source_type: "entry".into(),
                    source_id: entry.id.clone(),
                    text: chunks[0].text.clone(),
                    score: 0.61,
                },
                VectorMatch {
                    chunk_id: chunks[0].id.clone(),
                    source_type: "entry".into(),
                    source_id: entry.id.clone(),
                    text: "更强命中".into(),
                    score: 0.91,
                },
            ],
            8,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, "entry");
        assert_eq!(results[0].entity_id, entry.id);
        assert_eq!(results[0].title, "月光阔剑");
        assert_eq!(results[0].snippet, "更强命中");
        assert!((results[0].score - 0.91).abs() < 0.000_1);
    }
}
