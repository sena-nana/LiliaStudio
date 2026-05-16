use ameya_lib::db::{connection::open_database, migrations};

#[test]
fn runs_migrations_against_file_database() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("ameya-test.db");
    let mut connection = open_database(&db_path).expect("file database opens");

    migrations::run_migrations(&mut connection).expect("migrations run");

    assert_eq!(migrations::current_schema_version(&connection).unwrap(), 1);
}
