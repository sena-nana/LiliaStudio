use ameya_lib::{
    db::{connection::open_database, migrations},
    domain::{
        entry::{EntryDraft, EntryRepository},
        project::{ProjectDraft, ProjectRepository},
    },
};

#[test]
fn runs_migrations_against_file_database() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("ameya-test.db");
    let mut connection = open_database(&db_path).expect("file database opens");

    migrations::run_migrations(&mut connection).expect("migrations run");

    assert_eq!(migrations::current_schema_version(&connection).unwrap(), 3);
}

#[test]
fn rich_text_migration_normalizes_existing_plain_text_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("ameya-test.db");
    let mut connection = open_database(&db_path).expect("file database opens");

    migrations::run_migrations(&mut connection).expect("migrations run");
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "迁移项目".into(),
            description: String::new(),
        })
        .unwrap();
    let entry = EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "item".into(),
            title: "旧词条".into(),
            summary: "旧摘要".into(),
            body: "旧正文".into(),
            tags: vec![],
            status: "draft".into(),
        })
        .unwrap();
    connection
        .execute(
            "UPDATE entries SET summary = ?1, body = ?2 WHERE id = ?3",
            ("  旧摘要\n", "\n旧正文  ", &entry.id),
        )
        .unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 3", [])
        .unwrap();

    migrations::run_migrations(&mut connection).expect("rich text migration reruns");
    let migrated = EntryRepository::new(&connection)
        .get(&entry.id)
        .unwrap()
        .unwrap();

    assert!(migrated.summary.contains(r#""type":"doc""#));
    assert!(migrated.summary.contains(r#""text":"  旧摘要""#));
    assert!(migrated.body.contains(r#""text":"旧正文  ""#));
}
