use ameya_lib::{
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        event::{EventDraft, EventParticipantDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
        relation::{EntityRef, RelationDraft, RelationRepository},
    },
    services::relation_workspace::{relation_neighborhood, relation_suggestions},
    test_support::migrated_memory_database,
};

#[test]
fn relation_neighborhood_includes_forward_and_reverse_edges() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "关系项目".into(),
            description: String::new(),
        })
        .unwrap();
    let entries = EntryRepository::new(&connection);
    let relations = RelationRepository::new(&connection);
    let sword = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "月光阔剑".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let forge = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "technology".into(),
            title: "精灵锻造".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let city = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "潮汐城".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    relations
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef::entry(sword.id.clone()),
            target: EntityRef::entry(forge.id.clone()),
            relation_type: "来源于".into(),
            description: String::new(),
            confidence: 0.9,
            directed: true,
        })
        .unwrap();
    relations
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef::entry(city.id.clone()),
            target: EntityRef::entry(sword.id.clone()),
            relation_type: "发生于".into(),
            description: String::new(),
            confidence: 0.8,
            directed: true,
        })
        .unwrap();

    let neighborhood = relation_neighborhood(
        &connection,
        &project.id,
        EntityRef::entry(sword.id.clone()),
        1,
    )
    .unwrap();

    assert_eq!(neighborhood.relation_count, 2);
    assert_eq!(neighborhood.edges.len(), 2);
    assert!(neighborhood
        .nodes
        .iter()
        .any(|node| node.title == "精灵锻造"));
    assert!(neighborhood.nodes.iter().any(|node| node.title == "潮汐城"));
}

#[test]
fn relation_suggestions_filter_existing_relations_and_keep_structured_sources() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "建议项目".into(),
            description: String::new(),
        })
        .unwrap();
    let characters = CharacterRepository::new(&connection);
    let events = EventRepository::new(&connection);
    let relations = RelationRepository::new(&connection);
    let character = characters
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: "椎名".into(),
            aliases: vec![],
            summary: String::new(),
            appearance: String::new(),
            goals: String::new(),
            motivations: String::new(),
            fears: String::new(),
            faction: "观测者".into(),
            tags: vec![],
        })
        .unwrap();
    let event = events
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "围城战".into(),
                description: String::new(),
                time_label: "冬季".into(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 5,
                outcome: String::new(),
                tags: vec![],
            },
            vec![EventParticipantDraft {
                entity_type: "character".into(),
                entity_id: character.id.clone(),
                role: "defender".into(),
            }],
        )
        .unwrap();

    let first = relation_suggestions(
        &connection,
        &project.id,
        EntityRef {
            entity_type: "character".into(),
            entity_id: character.id.clone(),
        },
    )
    .unwrap();
    assert!(first
        .iter()
        .any(|suggestion| suggestion.target.entity_id == event.id
            && suggestion.relation_type == "参与事件"));

    relations
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef {
                entity_type: "character".into(),
                entity_id: character.id.clone(),
            },
            target: EntityRef {
                entity_type: "event".into(),
                entity_id: event.id.clone(),
            },
            relation_type: "参与事件".into(),
            description: String::new(),
            confidence: 1.0,
            directed: true,
        })
        .unwrap();
    let filtered = relation_suggestions(
        &connection,
        &project.id,
        EntityRef {
            entity_type: "character".into(),
            entity_id: character.id,
        },
    )
    .unwrap();
    assert!(!filtered
        .iter()
        .any(|suggestion| suggestion.target.entity_id == event.id
            && suggestion.relation_type == "参与事件"));
}

#[test]
fn relation_suggestions_include_axiom_source_without_embeddings() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "公理项目".into(),
            description: String::new(),
        })
        .unwrap();
    let entry = EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "world_rule".into(),
            title: "潮汐规则".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let axiom = AxiomRepository::new(&connection)
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "潮汐能".into(),
            predicate: "约束".into(),
            object: "满月稳定".into(),
            scope_time: String::new(),
            scope_location: String::new(),
            certainty: 0.9,
            source_entity_type: Some("entry".into()),
            source_entity_id: Some(entry.id.clone()),
            natural_language: "潮汐能只有在满月稳定。".into(),
            tags: vec![],
        })
        .unwrap();

    let suggestions = relation_suggestions(
        &connection,
        &project.id,
        EntityRef {
            entity_type: "entry".into(),
            entity_id: entry.id,
        },
    )
    .unwrap();

    assert!(suggestions
        .iter()
        .any(|suggestion| suggestion.target.entity_id == axiom.id
            && suggestion.relation_type == "支撑规则"));
}
