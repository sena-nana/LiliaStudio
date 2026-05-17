use ameya_lib::{
    db::{connection::open_memory_database, migrations::run_migrations},
    services::prompts::{
        copy_prompt_template, list_prompt_templates, preview_prompt_template,
        reset_builtin_prompt_templates, save_prompt_template, PromptTemplateDraft,
        PromptTemplatePreviewRequest, PromptTemplateVariableValue,
    },
};

#[test]
fn builtin_templates_reset_and_expose_variable_descriptions() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");

    let templates = list_prompt_templates(&connection).expect("templates");

    assert!(templates
        .iter()
        .any(|template| template.purpose == "completion_questions"));
    assert!(templates.iter().any(|template| {
        template
            .variables
            .iter()
            .any(|variable| variable.name == "project_context")
    }));

    let reset = reset_builtin_prompt_templates(&connection).expect("reset builtins");
    assert!(reset.iter().all(|template| template.built_in));
    assert!(reset.len() >= 4);
}

#[test]
fn users_can_copy_and_edit_templates_without_mutating_builtins() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let builtin = list_prompt_templates(&connection).expect("templates")[0].clone();

    let copy = copy_prompt_template(&connection, &builtin.id).expect("copy template");
    assert!(!copy.built_in);
    assert_ne!(copy.id, builtin.id);

    let saved = save_prompt_template(
        &connection,
        PromptTemplateDraft {
            id: Some(copy.id.clone()),
            name: "自定义审计".into(),
            purpose: "logic_audit".into(),
            template: "上下文：{{project_context}}\n目标：{{target_entity}}".into(),
        },
    )
    .expect("save template");

    assert_eq!(saved.name, "自定义审计");
    assert!(saved
        .variables
        .iter()
        .any(|variable| variable.name == "target_entity"));

    let builtins = list_prompt_templates(&connection).expect("templates");
    let original = builtins
        .iter()
        .find(|template| template.id == builtin.id)
        .expect("original builtin remains");
    assert!(original.built_in);
    assert_ne!(original.name, saved.name);
}

#[test]
fn prompt_preview_renders_known_values_and_reports_missing_variables() {
    let preview = preview_prompt_template(PromptTemplatePreviewRequest {
        template: "上下文：{{project_context}}\n问题：{{question}}\n目标：{{target_entity}}".into(),
        values: vec![
            PromptTemplateVariableValue {
                name: "project_context".into(),
                value: "北境粮食规则".into(),
            },
            PromptTemplateVariableValue {
                name: "question".into(),
                value: "是否冲突".into(),
            },
        ],
    });

    assert!(preview.prompt.contains("北境粮食规则"));
    assert!(preview.prompt.contains("是否冲突"));
    assert_eq!(preview.missing_variables, vec!["target_entity"]);
}
