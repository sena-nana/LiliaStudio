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
        character_growth::{apply_trait_delta, list_growth_workspace, CharacterTraitState, TraitDelta},
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
    let records = CharacterTraitDeltaRepository::new(&connection);
    records
        .create(CharacterTraitDeltaRecordDraft {
            project_id: project.id.clone(),
            character_id: character.id.clone(),
            source_event_id: event.id.clone(),
            trait_name: "responsibility".into(),
            delta: 0.7,
            reason: "保护平民".into(),
        })
        .unwrap();
    records
        .create(CharacterTraitDeltaRecordDraft {
            project_id: project.id.clone(),
            character_id: character.id.clone(),
            source_event_id: event.id,
            trait_name: "responsibility".into(),
            delta: 0.6,
            reason: "承担后果".into(),
        })
        .unwrap();

    let workspace = list_growth_workspace(&connection, &project.id).unwrap();
    let state = workspace.states.get(&character.id).unwrap();

    assert_eq!(state.values.get("responsibility").copied(), Some(1.0));
    assert_eq!(state.sources.len(), 2);
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
