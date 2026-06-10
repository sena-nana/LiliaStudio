use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::{
    axiom::{Axiom, AxiomRepository},
    character::CharacterRepository,
    entry::EntryRepository,
    entry_rich_text::entry_rich_text_to_plain_text,
    event::{Event, EventParticipant, EventRepository},
    relation::{EntityRef, Relation, RelationRepository},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntitySummary {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub subtitle: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelationSuggestion {
    pub source: EntityRef,
    pub target: EntityRef,
    pub relation_type: String,
    pub description: String,
    pub confidence: f64,
    pub directed: bool,
    pub reason: String,
    pub strength: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelationNeighborhood {
    pub center: EntitySummary,
    pub nodes: Vec<EntitySummary>,
    pub edges: Vec<Relation>,
    pub suggestions: Vec<RelationSuggestion>,
    pub missing: Vec<String>,
    pub relation_count: usize,
}

struct ProjectEntities {
    events: Vec<Event>,
    axioms: Vec<Axiom>,
    summaries: HashMap<String, EntitySummary>,
}

pub fn relation_neighborhood(
    connection: &Connection,
    project_id: &str,
    target: EntityRef,
    depth: usize,
) -> rusqlite::Result<RelationNeighborhood> {
    let entities = load_project_entities(connection, project_id)?;
    let center_key = entity_key(&target);
    let center = entities
        .summaries
        .get(&center_key)
        .cloned()
        .unwrap_or_else(|| missing_summary(&target));
    let relations = RelationRepository::new(connection).list_project(project_id)?;
    let edges = neighborhood_edges(&relations, &target, depth.clamp(1, 2));
    let mut node_keys = HashSet::from([center_key]);
    for edge in &edges {
        node_keys.insert(entity_key(&edge.source));
        node_keys.insert(entity_key(&edge.target));
    }
    let mut nodes = node_keys
        .into_iter()
        .map(|key| {
            entities
                .summaries
                .get(&key)
                .cloned()
                .unwrap_or_else(|| summary_from_key(&key))
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| {
        left.entity_type
            .cmp(&right.entity_type)
            .then_with(|| left.title.cmp(&right.title))
    });
    let relation_count = relations
        .iter()
        .filter(|relation| touches(relation, &target))
        .count();
    let suggestions = relation_suggestions(connection, project_id, target.clone())?;
    let missing = missing_items(&target, &center, relation_count, &suggestions);
    Ok(RelationNeighborhood {
        center,
        nodes,
        edges,
        suggestions,
        missing,
        relation_count,
    })
}

pub fn relation_suggestions(
    connection: &Connection,
    project_id: &str,
    target: EntityRef,
) -> rusqlite::Result<Vec<RelationSuggestion>> {
    let entities = load_project_entities(connection, project_id)?;
    let relations = RelationRepository::new(connection).list_project(project_id)?;
    let existing_pairs = existing_relation_pairs(&relations);
    let mut suggestions = Vec::new();

    add_event_participant_suggestions(
        connection,
        &target,
        &entities,
        &existing_pairs,
        &mut suggestions,
    )?;
    add_axiom_source_suggestions(&target, &entities, &existing_pairs, &mut suggestions);
    add_text_affinity_suggestions(&target, &entities, &existing_pairs, &mut suggestions);

    let mut seen = HashSet::new();
    suggestions.retain(|suggestion| {
        seen.insert(format!(
            "{}|{}|{}",
            entity_key(&suggestion.source),
            entity_key(&suggestion.target),
            suggestion.relation_type
        ))
    });
    suggestions.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    suggestions.truncate(8);
    Ok(suggestions)
}

pub fn relation_type_presets() -> Vec<String> {
    [
        "参与事件",
        "发生于",
        "属于阵营",
        "导致",
        "支撑规则",
        "约束",
        "来源于",
        "冲突候选",
        "语义相关",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn load_project_entities(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<ProjectEntities> {
    let entries = EntryRepository::new(connection).list_active(project_id)?;
    let characters = CharacterRepository::new(connection).list_active(project_id)?;
    let events = EventRepository::new(connection).list_active(project_id)?;
    let axioms = AxiomRepository::new(connection).list_active(project_id)?;
    let mut summaries = HashMap::new();

    for entry in &entries {
        summaries.insert(
            format!("entry:{}", entry.id),
            EntitySummary {
                entity_type: "entry".into(),
                entity_id: entry.id.clone(),
                title: entry.title.clone(),
                subtitle: entry.entry_type.clone(),
                summary: entry_rich_text_to_plain_text(&entry.summary),
            },
        );
    }
    for character in &characters {
        summaries.insert(
            format!("character:{}", character.id),
            EntitySummary {
                entity_type: "character".into(),
                entity_id: character.id.clone(),
                title: character.name.clone(),
                subtitle: character.faction.clone(),
                summary: character.summary.clone(),
            },
        );
    }
    for event in &events {
        summaries.insert(
            format!("event:{}", event.id),
            EntitySummary {
                entity_type: "event".into(),
                entity_id: event.id.clone(),
                title: event.title.clone(),
                subtitle: event.time_label.clone(),
                summary: event.description.clone(),
            },
        );
    }
    for axiom in &axioms {
        summaries.insert(
            format!("axiom:{}", axiom.id),
            EntitySummary {
                entity_type: "axiom".into(),
                entity_id: axiom.id.clone(),
                title: axiom.subject.clone(),
                subtitle: format!("{} = {}", axiom.predicate, axiom.object),
                summary: axiom.natural_language.clone(),
            },
        );
    }

    Ok(ProjectEntities {
        events,
        axioms,
        summaries,
    })
}

fn neighborhood_edges(relations: &[Relation], target: &EntityRef, depth: usize) -> Vec<Relation> {
    let mut selected = Vec::new();
    let mut frontier = HashSet::from([entity_key(target)]);
    let mut visited = HashSet::new();

    for _ in 0..depth {
        let mut next = HashSet::new();
        for relation in relations {
            let source_key = entity_key(&relation.source);
            let target_key = entity_key(&relation.target);
            if frontier.contains(&source_key) || frontier.contains(&target_key) {
                if visited.insert(relation.id.clone()) {
                    selected.push(relation.clone());
                }
                next.insert(source_key);
                next.insert(target_key);
            }
        }
        frontier = next;
    }
    selected
}

fn add_event_participant_suggestions(
    connection: &Connection,
    target: &EntityRef,
    entities: &ProjectEntities,
    existing_pairs: &HashSet<String>,
    suggestions: &mut Vec<RelationSuggestion>,
) -> rusqlite::Result<()> {
    let event_repository = EventRepository::new(connection);
    match target.entity_type.as_str() {
        "event" => {
            for participant in event_repository.list_participants(&target.entity_id)? {
                push_suggestion(
                    suggestions,
                    existing_pairs,
                    EntityRef {
                        entity_type: participant.entity_type,
                        entity_id: participant.entity_id,
                    },
                    target.clone(),
                    "参与事件",
                    "事件参与者",
                    0.95,
                    "高",
                );
            }
        }
        _ => {
            for event in &entities.events {
                if event_repository
                    .list_participants(&event.id)?
                    .into_iter()
                    .any(|participant| participant_matches(&participant, target))
                {
                    push_suggestion(
                        suggestions,
                        existing_pairs,
                        target.clone(),
                        EntityRef {
                            entity_type: "event".into(),
                            entity_id: event.id.clone(),
                        },
                        "参与事件",
                        "同一事件参与者",
                        0.95,
                        "高",
                    );
                }
            }
        }
    }
    Ok(())
}

fn add_axiom_source_suggestions(
    target: &EntityRef,
    entities: &ProjectEntities,
    existing_pairs: &HashSet<String>,
    suggestions: &mut Vec<RelationSuggestion>,
) {
    for axiom in &entities.axioms {
        let axiom_ref = EntityRef {
            entity_type: "axiom".into(),
            entity_id: axiom.id.clone(),
        };
        if target.entity_type == "axiom" && target.entity_id == axiom.id {
            if let (Some(source_type), Some(source_id)) =
                (&axiom.source_entity_type, &axiom.source_entity_id)
            {
                push_suggestion(
                    suggestions,
                    existing_pairs,
                    EntityRef {
                        entity_type: source_type.clone(),
                        entity_id: source_id.clone(),
                    },
                    axiom_ref,
                    "支撑规则",
                    "公理来源",
                    0.9,
                    "高",
                );
            }
        } else if axiom.source_entity_type.as_deref() == Some(target.entity_type.as_str())
            && axiom.source_entity_id.as_deref() == Some(target.entity_id.as_str())
        {
            push_suggestion(
                suggestions,
                existing_pairs,
                target.clone(),
                axiom_ref,
                "支撑规则",
                "公理来源",
                0.9,
                "高",
            );
        }
    }
}

fn add_text_affinity_suggestions(
    target: &EntityRef,
    entities: &ProjectEntities,
    existing_pairs: &HashSet<String>,
    suggestions: &mut Vec<RelationSuggestion>,
) {
    let Some(center) = entities.summaries.get(&entity_key(target)) else {
        return;
    };
    let center_tokens = semantic_tokens(&format!(
        "{} {} {}",
        center.title, center.subtitle, center.summary
    ));
    if center_tokens.is_empty() {
        return;
    }

    for candidate in entities.summaries.values() {
        if candidate.entity_type == target.entity_type && candidate.entity_id == target.entity_id {
            continue;
        }
        let candidate_tokens = semantic_tokens(&format!(
            "{} {} {}",
            candidate.title, candidate.subtitle, candidate.summary
        ));
        let overlap = center_tokens.intersection(&candidate_tokens).count();
        if overlap == 0 {
            continue;
        }
        let confidence = (0.55 + (overlap as f64 * 0.08)).min(0.82);
        push_suggestion(
            suggestions,
            existing_pairs,
            target.clone(),
            EntityRef {
                entity_type: candidate.entity_type.clone(),
                entity_id: candidate.entity_id.clone(),
            },
            "语义相关",
            "相关设定",
            confidence,
            if confidence >= 0.75 { "高" } else { "中" },
        );
    }
}

fn push_suggestion(
    suggestions: &mut Vec<RelationSuggestion>,
    existing_pairs: &HashSet<String>,
    source: EntityRef,
    target: EntityRef,
    relation_type: &str,
    reason: &str,
    confidence: f64,
    strength: &str,
) {
    if source.entity_id.is_empty() || target.entity_id.is_empty() || source == target {
        return;
    }
    if existing_pairs.contains(&relation_pair_key(&source, &target)) {
        return;
    }
    suggestions.push(RelationSuggestion {
        source,
        target,
        relation_type: relation_type.into(),
        description: reason.into(),
        confidence,
        directed: true,
        reason: reason.into(),
        strength: strength.into(),
    });
}

fn missing_items(
    target: &EntityRef,
    center: &EntitySummary,
    relation_count: usize,
    suggestions: &[RelationSuggestion],
) -> Vec<String> {
    let mut missing = Vec::new();
    if relation_count == 0 {
        missing.push("缺少已确认关系".into());
    }
    if center.summary.trim().is_empty() {
        missing.push("缺少摘要".into());
    }
    if target.entity_type == "character" && center.subtitle.trim().is_empty() {
        missing.push("缺少阵营".into());
    }
    if target.entity_type == "event" && center.subtitle.trim().is_empty() {
        missing.push("缺少时间".into());
    }
    if !suggestions.is_empty() {
        missing.push("存在待确认关联建议".into());
    }
    missing
}

fn existing_relation_pairs(relations: &[Relation]) -> HashSet<String> {
    relations
        .iter()
        .flat_map(|relation| {
            [
                relation_pair_key(&relation.source, &relation.target),
                relation_pair_key(&relation.target, &relation.source),
            ]
        })
        .collect()
}

fn relation_pair_key(source: &EntityRef, target: &EntityRef) -> String {
    format!("{}>{}", entity_key(source), entity_key(target))
}

fn entity_key(entity: &EntityRef) -> String {
    format!("{}:{}", entity.entity_type, entity.entity_id)
}

fn touches(relation: &Relation, entity: &EntityRef) -> bool {
    relation.source == *entity || relation.target == *entity
}

fn participant_matches(participant: &EventParticipant, target: &EntityRef) -> bool {
    participant.entity_type == target.entity_type && participant.entity_id == target.entity_id
}

fn missing_summary(entity: &EntityRef) -> EntitySummary {
    EntitySummary {
        entity_type: entity.entity_type.clone(),
        entity_id: entity.entity_id.clone(),
        title: entity.entity_id.clone(),
        subtitle: String::new(),
        summary: String::new(),
    }
}

fn summary_from_key(key: &str) -> EntitySummary {
    let mut parts = key.splitn(2, ':');
    let entity_type = parts.next().unwrap_or_default().to_string();
    let entity_id = parts.next().unwrap_or_default().to_string();
    EntitySummary {
        entity_type,
        entity_id: entity_id.clone(),
        title: entity_id,
        subtitle: String::new(),
        summary: String::new(),
    }
}

fn semantic_tokens(text: &str) -> HashSet<String> {
    text.split(|character: char| {
        character.is_whitespace()
            || matches!(
                character,
                ',' | '，' | '。' | '、' | ';' | '；' | ':' | '：' | '(' | ')' | '（' | '）'
            )
    })
    .map(str::trim)
    .filter(|token| token.chars().count() >= 2)
    .map(str::to_string)
    .collect()
}
