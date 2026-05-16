use ameya_lib::{
    domain::{
        entry::{EntryDraft, EntryRepository},
        project::{ProjectDraft, ProjectRepository},
    },
    services::rag::{build_context_pack, index_project_chunks, upsert_embedding, vector_search},
    test_support::migrated_memory_database,
};

#[test]
fn indexes_entry_chunks_and_builds_context_pack() {
    let connection = migrated_memory_database();
    let project = ProjectRepository::new(&connection)
        .create(ProjectDraft {
            name: "RAG 项目".into(),
            description: String::new(),
        })
        .unwrap();
    EntryRepository::new(&connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "world_rule".into(),
            title: "潮汐能规则".into(),
            summary: "潮汐能驱动月光装置".into(),
            body: "潮汐能只能在满月时稳定输出。".into(),
            tags: vec!["能源".into()],
            status: "draft".into(),
        })
        .unwrap();

    let chunks = index_project_chunks(&connection, &project.id, 64).unwrap();
    assert_eq!(chunks.len(), 1);
    upsert_embedding(&connection, &chunks[0].id, "test-embedding", vec![1.0, 0.0, 0.0]).unwrap();

    let matches = vector_search(&connection, &project.id, vec![1.0, 0.0, 0.0], 5).unwrap();
    assert_eq!(matches[0].chunk_id, chunks[0].id);

    let context = build_context_pack(&connection, &project.id, "潮汐能", vec![1.0, 0.0, 0.0]).unwrap();
    assert!(context.items[0].text.contains("潮汐能"));
    assert_eq!(context.items[0].source_type, "entry");
}
