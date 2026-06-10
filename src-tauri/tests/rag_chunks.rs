use ameya_lib::{
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        event::{EventDraft, EventRepository},
        project::{ProjectDraft, ProjectRepository},
    },
    services::rag::index_project_chunks,
    test_support::migrated_memory_database,
};
use rusqlite::params;
use std::collections::BTreeSet;

#[test]
fn indexes_chunks_for_all_document_sources_without_embeddings() {
    let connection = migrated_memory_database();
    let project_id = seed_project_documents(&connection).project_id;

    let chunks = index_project_chunks(&connection, &project_id, 600).expect("index succeeds");
    let source_types = chunks
        .iter()
        .map(|chunk| chunk.source_type.as_str())
        .collect::<BTreeSet<_>>();
    let all_text = chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let embedding_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
        .unwrap();

    assert_eq!(
        source_types,
        BTreeSet::from(["axiom", "character", "entry", "event"])
    );
    assert!(all_text.contains("潮汐城设定"));
    assert!(all_text.contains("姓名: 林澈"));
    assert!(all_text.contains("标题: 月门开启"));
    assert!(all_text.contains("主语: 月门"));
    assert!(chunks.iter().all(|chunk| chunk.estimated_tokens > 0));
    assert!(chunks.iter().all(|chunk| !chunk.updated_at.is_empty()));
    assert_eq!(embedding_count, 0);
}

#[test]
fn keeps_unchanged_chunks_and_refreshes_changed_content() {
    let connection = migrated_memory_database();
    let seeded = seed_project_documents(&connection);
    let first = index_project_chunks(&connection, &seeded.project_id, 600).expect("first index");
    let entry_chunk = first
        .iter()
        .find(|chunk| chunk.source_type == "entry")
        .expect("entry chunk exists");

    connection
        .execute(
            "UPDATE document_chunks SET updated_at = 'stable-marker' WHERE id = ?1",
            params![entry_chunk.id],
        )
        .unwrap();
    let second = index_project_chunks(&connection, &seeded.project_id, 600).expect("second index");
    let unchanged = second
        .iter()
        .find(|chunk| chunk.id == entry_chunk.id)
        .expect("same entry chunk remains");

    assert_eq!(first.len(), second.len());
    assert_eq!(unchanged.updated_at, "stable-marker");
    assert_eq!(unchanged.content_hash, entry_chunk.content_hash);

    EntryRepository::new(&connection)
        .update(
            &seeded.entry_id,
            EntryDraft {
                project_id: seeded.project_id.clone(),
                entry_type: "world".into(),
                title: "潮汐城设定".into(),
                summary: "海墙会在双月夜升起".into(),
                body: "月门钥匙改由林澈保管".into(),
                tags: vec!["潮汐".into()],
                status: "draft".into(),
            },
        )
        .unwrap();
    let third = index_project_chunks(&connection, &seeded.project_id, 600).expect("third index");
    let changed = third
        .iter()
        .find(|chunk| chunk.id == entry_chunk.id)
        .expect("entry chunk id remains stable");

    assert_ne!(changed.content_hash, entry_chunk.content_hash);
    assert_ne!(changed.updated_at, "stable-marker");
    assert!(changed.text.contains("林澈保管"));
}

#[test]
fn removes_chunks_for_deleted_sources() {
    let connection = migrated_memory_database();
    let seeded = seed_project_documents(&connection);
    index_project_chunks(&connection, &seeded.project_id, 600).expect("first index");

    CharacterRepository::new(&connection)
        .soft_delete(&seeded.character_id)
        .unwrap();
    let chunks = index_project_chunks(&connection, &seeded.project_id, 600).expect("reindex");

    assert!(chunks.iter().all(|chunk| chunk.source_type != "character"));
}

struct SeededDocuments {
    project_id: String,
    entry_id: String,
    character_id: String,
}

fn seed_project_documents(connection: &rusqlite::Connection) -> SeededDocuments {
    let project = ProjectRepository::new(connection)
        .create(ProjectDraft {
            name: "索引项目".into(),
            description: String::new(),
        })
        .unwrap();
    let entry = EntryRepository::new(connection)
        .create(EntryDraft {
            project_id: project.id.clone(),
            entry_type: "world".into(),
            title: "潮汐城设定".into(),
            summary: "海墙会在双月夜升起".into(),
            body: "月门钥匙由守门人保管".into(),
            tags: vec!["潮汐".into()],
            status: "draft".into(),
        })
        .unwrap();
    let character = CharacterRepository::new(connection)
        .create(CharacterDraft {
            project_id: project.id.clone(),
            name: "林澈".into(),
            aliases: vec!["守门人".into()],
            summary: "负责月门钥匙".into(),
            appearance: "银发".into(),
            goals: "守住潮汐城".into(),
            motivations: "保护居民".into(),
            fears: "月门失控".into(),
            faction: "守门会".into(),
            tags: vec!["主角".into()],
        })
        .unwrap();
    EventRepository::new(connection)
        .create(
            EventDraft {
                project_id: project.id.clone(),
                title: "月门开启".into(),
                description: "双月夜时月门短暂开启".into(),
                time_label: "双月夜".into(),
                sort_key: 10,
                start_label: "夜初".into(),
                end_label: "夜半".into(),
                location: "潮汐城".into(),
                importance: 4,
                outcome: "引出外海信使".into(),
                tags: vec!["主线".into()],
            },
            vec![],
        )
        .unwrap();
    AxiomRepository::new(connection)
        .create(AxiomDraft {
            project_id: project.id.clone(),
            subject: "月门".into(),
            predicate: "只能在".into(),
            object: "双月夜开启".into(),
            scope_time: "双月夜".into(),
            scope_location: "潮汐城".into(),
            certainty: 1.0,
            source_entity_type: Some("entry".into()),
            source_entity_id: Some(entry.id.clone()),
            natural_language: "月门只能在双月夜开启。".into(),
            tags: vec!["规则".into()],
        })
        .unwrap();

    SeededDocuments {
        project_id: project.id,
        entry_id: entry.id,
        character_id: character.id,
    }
}
