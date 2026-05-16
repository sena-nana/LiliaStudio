use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::domain::{
    axiom::{Axiom, AxiomDraft, AxiomRepository},
    character::{Character, CharacterDraft, CharacterRepository},
    entry::{Entry, EntryDraft, EntryRepository},
    event::{Event, EventDraft, EventRepository},
    project::{Project, ProjectDraft, ProjectRepository},
    relation::{Relation, RelationDraft, RelationRepository},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchive {
    pub version: u32,
    pub project: Project,
    pub entries: Vec<Entry>,
    pub characters: Vec<Character>,
    pub events: Vec<Event>,
    pub axioms: Vec<Axiom>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportedProject {
    pub project: Project,
}

pub fn export_project(connection: &Connection, project_id: &str) -> rusqlite::Result<ProjectArchive> {
    let project = ProjectRepository::new(connection)
        .get(project_id)?
        .expect("project should exist for export");
    Ok(ProjectArchive {
        version: 1,
        entries: EntryRepository::new(connection).list_active(project_id)?,
        characters: CharacterRepository::new(connection).list_active(project_id)?,
        events: EventRepository::new(connection).list_active(project_id)?,
        axioms: AxiomRepository::new(connection).list_active(project_id)?,
        relations: RelationRepository::new(connection).list_project(project_id)?,
        project,
    })
}

pub fn import_project(
    connection: &Connection,
    archive: ProjectArchive,
) -> rusqlite::Result<ImportedProject> {
    let project = ProjectRepository::new(connection).create_with_name_suffix(
        ProjectDraft {
            name: archive.project.name,
            description: archive.project.description,
        },
        " 副本",
    )?;

    let entries = EntryRepository::new(connection);
    for entry in archive.entries {
        entries.create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: entry.entry_type,
            title: entry.title,
            summary: entry.summary,
            body: entry.body,
            tags: entry.tags,
            status: entry.status,
        })?;
    }

    let characters = CharacterRepository::new(connection);
    for character in archive.characters {
        characters.create(CharacterDraft {
            project_id: project.id.clone(),
            name: character.name,
            aliases: character.aliases,
            summary: character.summary,
            appearance: character.appearance,
            goals: character.goals,
            motivations: character.motivations,
            fears: character.fears,
            faction: character.faction,
            tags: character.tags,
        })?;
    }

    let events = EventRepository::new(connection);
    for event in archive.events {
        events.create(
            EventDraft {
                project_id: project.id.clone(),
                title: event.title,
                description: event.description,
                time_label: event.time_label,
                sort_key: event.sort_key,
                start_label: event.start_label,
                end_label: event.end_label,
                location: event.location,
                importance: event.importance,
                outcome: event.outcome,
                tags: event.tags,
            },
            vec![],
        )?;
    }

    let axioms = AxiomRepository::new(connection);
    for axiom in archive.axioms {
        axioms.create(AxiomDraft {
            project_id: project.id.clone(),
            subject: axiom.subject,
            predicate: axiom.predicate,
            object: axiom.object,
            scope_time: axiom.scope_time,
            scope_location: axiom.scope_location,
            certainty: axiom.certainty,
            source_entity_type: axiom.source_entity_type,
            source_entity_id: axiom.source_entity_id,
            natural_language: axiom.natural_language,
            tags: axiom.tags,
        })?;
    }

    let relations = RelationRepository::new(connection);
    for relation in archive.relations {
        relations.create(RelationDraft {
            project_id: project.id.clone(),
            source: relation.source,
            target: relation.target,
            relation_type: relation.relation_type,
            description: relation.description,
            confidence: relation.confidence,
            directed: relation.directed,
        })?;
    }

    Ok(ImportedProject { project })
}
