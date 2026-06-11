use ameya_lib::{
    domain::{
        character::{CharacterDraft, CharacterRepository},
        character_trait_delta::{CharacterTraitDeltaRecordDraft, CharacterTraitDeltaRepository},
        event::{EventDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
    },
    logic::{
        conflict::{detect_conflicts, Fact},
        quickxplain::minimal_conflict,
        repair::suggest_repairs,
    },
    services::{
        character_growth::{
            apply_trait_delta, create_trait_delta_record, list_growth_workspace,
            CharacterTraitState, TraitDelta,
        },
        simulation::simulate_scenario,
    },
    test_support::migrated_memory_database,
};

#[test]
fn detects_conflicting_axiom_facts_and_suggests_repairs() {
    let facts = vec![
        Fact::axiom("a1", "月光金属", "state", "solid", "第三纪", "北方"),
        Fact::axiom("a2", "月光金属", "state", "liquid", "第三纪", "北方"),
    ];

    let conflicts = detect_conflicts(&facts);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].fact_ids, vec!["a1", "a2"]);

    let minimal = minimal_conflict(&facts, detect_conflicts).unwrap();
    assert_eq!(minimal.len(), 2);
    let repairs = suggest_repairs(&conflicts[0]);
    assert!(repairs
        .iter()
        .any(|repair| repair.title.contains("添加例外")));
}

#[test]
fn applies_character_trait_deltas_with_source_trace() {
    let mut state = CharacterTraitState::default();
    apply_trait_delta(
        &mut state,
        TraitDelta {
            source_event_id: "event_1".into(),
            trait_name: "responsibility".into(),
            delta: 0.35,
            reason: "围城战中保护平民".into(),
        },
    );

    assert_eq!(state.values.get("responsibility").copied(), Some(0.35));
    assert_eq!(state.sources[0].source_event_id, "event_1");
}

#[test]
fn builds_character_trait_state_from_saved_records() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "project".into(),
            description: String::new(),
        })
        .unwrap();
    let character = CharacterRepository::new(&connection)
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
    let event = EventRepository::new(&connection)
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "围城战".into(),
                description: String::new(),
                time_label: String::new(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 5,
                outcome: String::new(),
                tags: vec![],
            },
            vec![],
        )
        .unwrap();
    create_trait_delta_record(
        &connection,
        CharacterTraitDeltaRecordDraft {
            project_id: project.id.clone(),
            character_id: character.id.clone(),
            source_event_id: event.id.clone(),
            trait_name: "responsibility".into(),
            delta: 0.7,
            reason: "保护平民".into(),
        },
    )
    .unwrap();
    create_trait_delta_record(
        &connection,
        CharacterTraitDeltaRecordDraft {
            project_id: project.id.clone(),
            character_id: character.id.clone(),
            source_event_id: event.id,
            trait_name: "responsibility".into(),
            delta: 0.6,
            reason: "承担后果".into(),
        },
    )
    .unwrap();

    let workspace = list_growth_workspace(&connection, &project.id).unwrap();
    let state = workspace.states.get(&character.id).unwrap();

    assert_eq!(state.values.get("responsibility").copied(), Some(1.0));
    assert_eq!(state.sources.len(), 2);
}

#[test]
fn rejects_trait_delta_records_with_inactive_or_cross_project_links() {
    let connection = migrated_memory_database();
    let first = create_project_character_and_event(&connection, "first");
    let second = create_project_character_and_event(&connection, "second");

    let mut cross_project_event = trait_delta_draft(&first, "responsibility", 0.3, "跨项目事件");
    cross_project_event.source_event_id = second.event_id.clone();
    assert!(create_trait_delta_record(&connection, cross_project_event).is_err());
    assert!(list_growth_workspace(&connection, &first.project_id)
        .unwrap()
        .records
        .is_empty());

    CharacterRepository::new(&connection)
        .soft_delete(&first.character_id)
        .unwrap();
    assert!(create_trait_delta_record(
        &connection,
        trait_delta_draft(&first, "responsibility", 0.3, "已删除角色"),
    )
    .is_err());

    EventRepository::new(&connection)
        .soft_delete(&second.event_id)
        .unwrap();
    assert!(create_trait_delta_record(
        &connection,
        trait_delta_draft(&second, "resolve", 0.2, "已删除事件"),
    )
    .is_err());
}

#[test]
fn growth_workspace_ignores_records_for_deleted_characters_or_events() {
    let connection = migrated_memory_database();
    let project = create_project_character_and_event(&connection, "project");
    CharacterTraitDeltaRepository::new(&connection)
        .create(trait_delta_draft(
            &project,
            "responsibility",
            0.5,
            "保护平民",
        ))
        .unwrap();

    let workspace = list_growth_workspace(&connection, &project.project_id).unwrap();
    assert_eq!(workspace.records.len(), 1);

    EventRepository::new(&connection)
        .soft_delete(&project.event_id)
        .unwrap();
    let workspace = list_growth_workspace(&connection, &project.project_id).unwrap();
    assert!(workspace.records.is_empty());
    assert!(workspace.states.is_empty());

    let project = create_project_character_and_event(&connection, "second");
    CharacterTraitDeltaRepository::new(&connection)
        .create(trait_delta_draft(&project, "resolve", 0.5, "承担后果"))
        .unwrap();
    CharacterRepository::new(&connection)
        .soft_delete(&project.character_id)
        .unwrap();

    let workspace = list_growth_workspace(&connection, &project.project_id).unwrap();
    assert!(workspace.records.is_empty());
    assert!(workspace.states.is_empty());
}

#[test]
fn simulation_report_is_structured_without_ai() {
    let report = simulate_scenario(
        "project_1",
        "如果北方发生饥荒",
        vec!["粮食".into(), "北方城墙".into()],
    );

    assert_eq!(report.project_id, "project_1");
    assert!(report.phases[0].summary.contains("如果北方发生饥荒"));
    assert!(!report.risks.is_empty());
}

struct GrowthFixture {
    project_id: String,
    character_id: String,
    event_id: String,
}

fn create_project_character_and_event(
    connection: &rusqlite::Connection,
    name: &str,
) -> GrowthFixture {
    let project = ProjectRepository::new(connection)
        .create(ProjectDraft {
            name: name.into(),
            description: String::new(),
        })
        .unwrap();
    let character = CharacterRepository::new(connection)
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: format!("{name} character"),
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
    let event = EventRepository::new(connection)
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: format!("{name} event"),
                description: String::new(),
                time_label: String::new(),
                sort_key: 1,
                start_label: String::new(),
                end_label: String::new(),
                location: String::new(),
                importance: 5,
                outcome: String::new(),
                tags: vec![],
            },
            vec![],
        )
        .unwrap();

    GrowthFixture {
        project_id: project.id,
        character_id: character.id,
        event_id: event.id,
    }
}

fn trait_delta_draft(
    fixture: &GrowthFixture,
    trait_name: &str,
    delta: f32,
    reason: &str,
) -> CharacterTraitDeltaRecordDraft {
    CharacterTraitDeltaRecordDraft {
        project_id: fixture.project_id.clone(),
        character_id: fixture.character_id.clone(),
        source_event_id: fixture.event_id.clone(),
        trait_name: trait_name.into(),
        delta,
        reason: reason.into(),
    }
}
