use ameya_lib::{
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        event::{EventDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
        relation::{EntityRef, RelationDraft, RelationRepository},
    },
    services::{
        import_export::{export_project, import_project},
        search::{search_project, SearchFilter},
    },
    test_support::migrated_memory_database,
};

#[test]
fn project_search_finds_matches_across_core_entities() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "雨夜都市".into(),
            description: String::new(),
        })
        .unwrap();

    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "月光阔剑".into(),
            summary: "潮汐能武器".into(),
            body: "由精灵锻造技艺制造。".into(),
            tags: vec!["武器".into()],
            status: "draft".into(),
        })
        .unwrap();
    CharacterRepository::new(&connection)
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: "潮汐观测者".into(),
            aliases: vec![],
            summary: "研究月光阔剑的角色".into(),
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
                description: "月光阔剑首次被公开使用。".into(),
                time_label: "第三纪".into(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 4,
                outcome: String::new(),
                tags: vec![],
            },
            vec![],
        )
        .unwrap();
    AxiomRepository::new(&connection)
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "潮汐能".into(),
            predicate: "powers".into(),
            object: "月光阔剑".into(),
            scope_time: String::new(),
            scope_location: String::new(),
            certainty: 1.0,
            source_entity_type: None,
            source_entity_id: None,
            natural_language: "潮汐能驱动月光阔剑。".into(),
            tags: vec![],
        })
        .unwrap();

    let results = search_project(
        &connection,
        SearchFilter {
            project_id: project.id,
            query: "月光".into(),
            entity_types: vec![],
        },
    )
    .unwrap();

    let result_types: Vec<_> = results.iter().map(|result| result.entity_type.as_str()).collect();
    assert!(result_types.contains(&"entry"));
    assert!(result_types.contains(&"character"));
    assert!(result_types.contains(&"event"));
    assert!(result_types.contains(&"axiom"));
}

#[test]
fn export_and_import_project_creates_a_new_project_copy() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "原始项目".into(),
            description: "可导出".into(),
        })
        .unwrap();

    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "北方城墙".into(),
            summary: String::new(),
            body: "围城战发生地。".into(),
            tags: vec!["地点".into()],
            status: "draft".into(),
        })
        .unwrap();

    let archive = export_project(&connection, &project.id).unwrap();
    let imported = import_project(&connection, archive).unwrap();

    assert_ne!(imported.project.id, project.id);
    assert!(imported.project.name.contains("原始项目"));
    assert_eq!(EntryRepository::new(&connection).list_active(&imported.project.id).unwrap().len(), 1);
}

#[test]
fn import_project_remaps_relation_entity_ids_to_imported_records() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "原始项目".into(),
            description: "可导出".into(),
        })
        .unwrap();
    let entries = EntryRepository::new(&connection);
    let source = entries
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
    let target = entries
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
    RelationRepository::new(&connection)
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef::entry(source.id.clone()),
            target: EntityRef::entry(target.id.clone()),
            relation_type: "derived_from".into(),
            description: String::new(),
            confidence: 1.0,
            directed: true,
        })
        .unwrap();

    let archive = export_project(&connection, &project.id).unwrap();
    let imported = import_project(&connection, archive).unwrap();
    let imported_entries = EntryRepository::new(&connection)
        .list_active(&imported.project.id)
        .unwrap();
    let imported_relations = RelationRepository::new(&connection)
        .list_project(&imported.project.id)
        .unwrap();

    assert_eq!(imported_relations.len(), 1);
    assert_ne!(imported_relations[0].source.entity_id, source.id);
    assert_ne!(imported_relations[0].target.entity_id, target.id);
    assert!(imported_entries
        .iter()
        .any(|entry| entry.id == imported_relations[0].source.entity_id));
    assert!(imported_entries
        .iter()
        .any(|entry| entry.id == imported_relations[0].target.entity_id));
}
