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
        rag::{index_project_chunks, upsert_embedding},
        search::{search_project, semantic_search_results, SearchFilter},
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

    let result_types: Vec<_> = results
        .iter()
        .map(|result| result.entity_type.as_str())
        .collect();
    assert!(result_types.contains(&"entry"));
    assert!(result_types.contains(&"character"));
    assert!(result_types.contains(&"event"));
    assert!(result_types.contains(&"axiom"));
}

#[test]
fn project_search_finds_text_inside_entry_rich_text_json() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "富文本项目".into(),
            description: String::new(),
        })
        .unwrap();
    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "world_rule".into(),
            title: "规则".into(),
            summary: r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"潮汐规则"}]}]}"#.into(),
            body: r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"月光阔剑只能在涨潮时启动"}]}]}"#.into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();

    let results = search_project(
        &connection,
        SearchFilter {
            project_id: project.id,
            query: "涨潮".into(),
            entity_types: vec!["entry".into()],
        },
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].snippet.contains("涨潮"));
    assert!(!results[0].snippet.contains("type"));
}

#[test]
fn project_search_finds_text_inside_entry_code_blocks() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "代码块搜索项目".into(),
            description: String::new(),
        })
        .unwrap();
    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "world_rule".into(),
            title: "规则".into(),
            summary: String::new(),
            body: r#"{"type":"doc","content":[{"type":"codeBlock","content":[{"type":"text","text":"moon_gate opens at high tide"}]}]}"#.into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();

    let results = search_project(
        &connection,
        SearchFilter {
            project_id: project.id,
            query: "high tide".into(),
            entity_types: vec!["entry".into()],
        },
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].snippet.contains("high tide"));
    assert!(!results[0].snippet.contains("codeBlock"));
}

#[test]
fn project_search_matches_entry_title_body_and_tags_case_insensitively() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "大小写搜索项目".into(),
            description: String::new(),
        })
        .unwrap();
    let entries = EntryRepository::new(&connection);
    let title_entry = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "Moon Gate".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let body_entry = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "rule".into(),
            title: "规则".into(),
            summary: String::new(),
            body: "Opens during Moonrise".into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let tag_entry = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "银钥匙".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec!["MoonKey".into()],
            status: "draft".into(),
        })
        .unwrap();

    let results = search_project(
        &connection,
        SearchFilter {
            project_id: project.id,
            query: "moon".into(),
            entity_types: vec!["entry".into()],
        },
    )
    .unwrap();

    let result_ids = results
        .iter()
        .map(|result| result.entity_id.as_str())
        .collect::<Vec<_>>();
    assert!(result_ids.contains(&title_entry.id.as_str()));
    assert!(result_ids.contains(&body_entry.id.as_str()));
    assert!(result_ids.contains(&tag_entry.id.as_str()));
}

#[test]
fn semantic_search_returns_entity_level_results_sorted_by_similarity() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "语义项目".into(),
            description: String::new(),
        })
        .unwrap();
    let first = EntryRepository::new(&connection)
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
    let second = EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "潮汐城".into(),
            summary: "环潮海港".into(),
            body: "月门会在此开启。".into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();

    let chunks = index_project_chunks(&connection, &project.id, 64).unwrap();
    for chunk in &chunks {
        if chunk.source_id == first.id {
            upsert_embedding(&connection, &chunk.id, "story-embed", vec![1.0, 0.0]).unwrap();
        } else if chunk.source_id == second.id {
            upsert_embedding(&connection, &chunk.id, "story-embed", vec![0.0, 1.0]).unwrap();
        }
    }

    let candidates =
        ameya_lib::services::rag::load_vector_candidates(&connection, &project.id).unwrap();
    let matches =
        ameya_lib::services::rag::rank_vector_candidates(candidates, vec![1.0, 0.0], 8);
    let results = semantic_search_results(&connection, matches, 8).unwrap();

    assert_eq!(results[0].entity_id, first.id);
    assert_eq!(results[0].title, "月光阔剑");
    assert_eq!(results[1].entity_id, second.id);
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
    assert_eq!(archive.version, 2);
    let imported = import_project(&connection, archive).unwrap();

    assert_ne!(imported.project.id, project.id);
    assert!(imported.project.name.contains("原始项目"));
    assert_eq!(
        EntryRepository::new(&connection)
            .list_active(&imported.project.id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn import_project_upgrades_v1_plain_text_entries_to_rich_text_json() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "原始项目".into(),
            description: String::new(),
        })
        .unwrap();
    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "北方城墙".into(),
            summary: "重要地点".into(),
            body: "围城战发生地。".into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();

    let mut archive = export_project(&connection, &project.id).unwrap();
    archive.version = 1;
    archive.entries[0].summary = "  旧摘要\n".into();
    archive.entries[0].body = "\n旧正文  ".into();
    let imported = import_project(&connection, archive).unwrap();
    let imported_entry = EntryRepository::new(&connection)
        .list_active(&imported.project.id)
        .unwrap()
        .remove(0);

    assert!(imported_entry.summary.contains(r#""type":"doc""#));
    assert!(imported_entry.summary.contains(r#""text":"  旧摘要""#));
    assert!(imported_entry.body.contains(r#""text":"旧正文  ""#));
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
