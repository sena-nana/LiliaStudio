use ameya_lib::{
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        event::{EventDraft, EventParticipantDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
        relation::{EntityRef, RelationDraft, RelationRepository},
    },
    test_support::migrated_memory_database,
};

#[test]
fn project_repository_creates_lists_updates_and_archives_projects() {
    let connection = migrated_memory_database();
    let repository = ProjectRepository::new(&connection);

    let project = repository
        .create(ProjectDraft {
            name: "雨夜都市".into(),
            description: "赛博幻想城市".into(),
        })
        .expect("project is created");

    assert_eq!(project.name, "雨夜都市");
    assert_eq!(repository.list_active().unwrap().len(), 1);

    let renamed = repository
        .update(
            &project.id,
            ProjectDraft {
                name: "雨夜都市 Revised".into(),
                description: "更新后的描述".into(),
            },
        )
        .expect("project is updated");
    assert_eq!(renamed.name, "雨夜都市 Revised");

    repository.archive(&project.id).expect("project is archived");
    assert!(repository.list_active().unwrap().is_empty());
    assert_eq!(repository.list_all().unwrap().len(), 1);
}

#[test]
fn entry_repository_soft_deletes_and_restores_entries() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let entries = EntryRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();

    let entry = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "月光阔剑".into(),
            summary: "潮汐能武器".into(),
            body: "由精灵锻造技艺制造。".into(),
            tags: vec!["武器".into(), "月光".into()],
            status: "draft".into(),
        })
        .unwrap();

    assert_eq!(entries.list_active(&project.id).unwrap().len(), 1);
    entries.soft_delete(&entry.id).unwrap();
    assert!(entries.list_active(&project.id).unwrap().is_empty());
    entries.restore(&entry.id).unwrap();
    assert_eq!(entries.get(&entry.id).unwrap().unwrap().title, "月光阔剑");
}

#[test]
fn character_repository_soft_deletes_characters() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let characters = CharacterRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();

    let character = characters
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: "椎名".into(),
            aliases: vec!["雨夜见证者".into()],
            summary: "冷静的调查者".into(),
            appearance: "黑发".into(),
            goals: "查明城市规则".into(),
            motivations: "保护同伴".into(),
            fears: "记忆丢失".into(),
            faction: "观测者".into(),
            tags: vec!["主角".into()],
        })
        .unwrap();

    assert_eq!(characters.list_active(&project.id).unwrap().len(), 1);
    characters.soft_delete(&character.id).unwrap();
    assert!(characters.list_active(&project.id).unwrap().is_empty());
}

#[test]
fn character_repository_updates_characters() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let characters = CharacterRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();

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
            faction: String::new(),
            tags: vec![],
        })
        .unwrap();

    let updated = characters
        .update(
            &character.id,
            CharacterDraft {
                project_id: project.id,
                name: "椎名 Revised".into(),
                aliases: vec!["观测者".into()],
                summary: "更新摘要".into(),
                appearance: "黑发".into(),
                goals: "找回记忆".into(),
                motivations: "保护同伴".into(),
                fears: "遗忘".into(),
                faction: "雨夜".into(),
                tags: vec!["主角".into()],
            },
        )
        .unwrap();

    assert_eq!(updated.name, "椎名 Revised");
    assert_eq!(updated.aliases, vec!["观测者"]);
    assert_eq!(updated.faction, "雨夜");
}

#[test]
fn event_repository_stores_participants() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let characters = CharacterRepository::new(&connection);
    let events = EventRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
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
            faction: String::new(),
            tags: vec![],
        })
        .unwrap();

    let event = events
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "围城战".into(),
                description: "城市被封锁三日。".into(),
                time_label: "第三纪 117 年".into(),
                sort_key: 117000,
                start_label: "第三纪 117 年 春".into(),
                end_label: "第三纪 117 年 夏".into(),
                location: "北方城墙".into(),
                importance: 5,
                outcome: "角色责任感上升".into(),
                tags: vec!["战争".into()],
            },
            vec![EventParticipantDraft {
                entity_type: "character".into(),
                entity_id: character.id.clone(),
                role: "defender".into(),
            }],
        )
        .unwrap();

    let participants = events.list_participants(&event.id).unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].entity_id, character.id);
}

#[test]
fn event_repository_updates_events_and_participants() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let characters = CharacterRepository::new(&connection);
    let events = EventRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
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
            faction: String::new(),
            tags: vec![],
        })
        .unwrap();
    let event = events
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "旧事件".into(),
                description: String::new(),
                time_label: String::new(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 1,
                outcome: String::new(),
                tags: vec![],
            },
            vec![],
        )
        .unwrap();

    let updated = events
        .update(
            &event.id,
            EventDraft {
                project_id: project.id,
                title: "新事件".into(),
                description: "更新描述".into(),
                time_label: "第三纪".into(),
                sort_key: 2,
                start_label: "春".into(),
                end_label: "夏".into(),
                location: "北方".into(),
                importance: 5,
                outcome: "责任感上升".into(),
                tags: vec!["战争".into()],
            },
            vec![EventParticipantDraft {
                entity_type: "character".into(),
                entity_id: character.id.clone(),
                role: "defender".into(),
            }],
        )
        .unwrap();

    assert_eq!(updated.title, "新事件");
    assert_eq!(events.list_participants(&updated.id).unwrap()[0].entity_id, character.id);
}

#[test]
fn relation_repository_lists_backlinks() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let entries = EntryRepository::new(&connection);
    let relations = RelationRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
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

    relations
        .create(RelationDraft {
            project_id: project.id,
            source: EntityRef::entry(sword.id),
            target: EntityRef::entry(forge.id.clone()),
            relation_type: "derived_from".into(),
            description: "武器来源于锻造技术".into(),
            confidence: 0.9,
            directed: true,
        })
        .unwrap();

    let backlinks = relations.list_backlinks(&EntityRef::entry(forge.id)).unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].relation_type, "derived_from");
}

#[test]
fn relation_repository_updates_lists_and_soft_deletes_relations() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let entries = EntryRepository::new(&connection);
    let relations = RelationRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
    let source = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "源".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let target = entries
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "location".into(),
            title: "目标".into(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    let relation = relations
        .create(RelationDraft {
            project_id: project.id.clone(),
            source: EntityRef::entry(source.id.clone()),
            target: EntityRef::entry(target.id.clone()),
            relation_type: "old".into(),
            description: String::new(),
            confidence: 0.5,
            directed: true,
        })
        .unwrap();

    let updated = relations
        .update(
            &relation.id,
            RelationDraft {
                project_id: project.id.clone(),
                source: EntityRef::entry(source.id),
                target: EntityRef::entry(target.id),
                relation_type: "new".into(),
                description: "更新关系".into(),
                confidence: 0.9,
                directed: false,
            },
        )
        .unwrap();
    assert_eq!(updated.relation_type, "new");
    assert_eq!(relations.list_project(&project.id).unwrap().len(), 1);

    relations.soft_delete(&relation.id).unwrap();
    assert!(relations.list_project(&project.id).unwrap().is_empty());
}

#[test]
fn axiom_repository_searches_by_subject_and_predicate() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let axioms = AxiomRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();

    axioms
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "月光金属".into(),
            predicate: "state_below_1000c".into(),
            object: "solid".into(),
            scope_time: "第三纪".into(),
            scope_location: "北方".into(),
            certainty: 0.95,
            source_entity_type: None,
            source_entity_id: None,
            natural_language: "月光金属在 1000 度以下保持固体。".into(),
            tags: vec!["物理".into()],
        })
        .unwrap();

    let matches = axioms.search(&project.id, "月光").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].predicate, "state_below_1000c");
}

#[test]
fn axiom_repository_updates_and_soft_deletes_axioms() {
    let connection = migrated_memory_database();
    let projects = ProjectRepository::new(&connection);
    let axioms = AxiomRepository::new(&connection);
    let project = projects
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
    let axiom = axioms
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "旧主体".into(),
            predicate: "defines".into(),
            object: "旧对象".into(),
            scope_time: String::new(),
            scope_location: String::new(),
            certainty: 1.0,
            source_entity_type: None,
            source_entity_id: None,
            natural_language: String::new(),
            tags: vec![],
        })
        .unwrap();

    let updated = axioms
        .update(
            &axiom.id,
            AxiomDraft {
                project_id: project.id.clone(),
                subject: "新主体".into(),
                predicate: "requires".into(),
                object: "新对象".into(),
                scope_time: "第三纪".into(),
                scope_location: "北方".into(),
                certainty: 0.8,
                source_entity_type: Some("entry".into()),
                source_entity_id: Some("entry_1".into()),
                natural_language: "新主体需要新对象。".into(),
                tags: vec!["规则".into()],
            },
        )
        .unwrap();

    assert_eq!(updated.subject, "新主体");
    assert_eq!(updated.certainty, 0.8);

    axioms.soft_delete(&axiom.id).unwrap();
    assert!(axioms.list_active(&project.id).unwrap().is_empty());
}
