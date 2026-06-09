use ameya_lib::{
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        event::{EventDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
        relation::{EntityRef, RelationDraft, RelationRepository},
    },
    services::library::load_project_snapshot,
    test_support::migrated_memory_database,
};

#[test]
fn library_project_snapshot_returns_core_collections_in_one_read() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "雨夜都市".into(),
            description: String::new(),
        })
        .unwrap();
    let entry = EntryRepository::new(&connection)
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
    CharacterRepository::new(&connection)
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: "椎名".into(),
            aliases: vec![],
            summary: String::new(),
            appearance: String::new(),
            goals: String::new(),
            motivations: String::new(),
            fears: String::new(),
            faction: String::new(),
            tags: vec![],
        })
        .unwrap();
    EventRepository::new(&connection)
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "围城战".into(),
                description: String::new(),
                time_label: "第三纪".into(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 3,
                outcome: String::new(),
                tags: vec![],
            },
            vec![],
        )
        .unwrap();
    AxiomRepository::new(&connection)
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "月光金属".into(),
            predicate: "defines".into(),
            object: "稀有".into(),
            scope_time: String::new(),
            scope_location: String::new(),
            certainty: 1.0,
            source_entity_type: None,
            source_entity_id: None,
            natural_language: String::new(),
            tags: vec![],
        })
        .unwrap();
    RelationRepository::new(&connection)
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef::entry(entry.id.clone()),
            target: EntityRef::entry(entry.id.clone()),
            relation_type: "self_reference".into(),
            description: String::new(),
            confidence: 1.0,
            directed: true,
        })
        .unwrap();

    let snapshot = load_project_snapshot(&connection, &project.id).unwrap();

    assert_eq!(snapshot.project_id, project.id);
    assert_eq!(snapshot.entries.len(), 1);
    assert_eq!(snapshot.characters.len(), 1);
    assert_eq!(snapshot.events.len(), 1);
    assert_eq!(snapshot.axioms.len(), 1);
    assert_eq!(snapshot.relations.len(), 1);
}
