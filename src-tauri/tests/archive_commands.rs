use rusqlite::{params, Connection};

use context_vault_lib::commands::archive::{list_conversations, ArchiveQuery};
use context_vault_lib::db::migrations::run_migrations;

fn seed_archive_db() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    run_migrations(&connection).unwrap();

    connection
        .execute(
            "INSERT INTO conversations (
                id, source_app, source_conversation_id, workspace_id, title, subtitle,
                created_at, updated_at, sync_strength, status, raw_metadata_json
            ) VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, 'ready', '{}')",
            params![
                "conv_1",
                "claude_code",
                "source_1",
                "Older conversation",
                100_i64,
                100_i64,
                "partial"
            ],
        )
        .unwrap();

    connection
        .execute(
            "INSERT INTO conversations (
                id, source_app, source_conversation_id, workspace_id, title, subtitle,
                created_at, updated_at, sync_strength, status, raw_metadata_json
            ) VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, 'ready', '{}')",
            params![
                "conv_2",
                "claude_code",
                "source_2",
                "gRPC 连接池耗尽排查",
                200_i64,
                200_i64,
                "full"
            ],
        )
        .unwrap();

    connection
}

#[test]
fn list_conversations_returns_rows_sorted_by_updated_at() {
    let db = seed_archive_db();
    let rows = list_conversations(&db, ArchiveQuery::default()).unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].title, "gRPC 连接池耗尽排查");
}

#[test]
fn list_conversations_filters_by_sync_strength() {
    let db = seed_archive_db();
    let rows = list_conversations(
        &db,
        ArchiveQuery {
            sync_strength: Some("full".into()),
            ..ArchiveQuery::default()
        },
    )
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "gRPC 连接池耗尽排查");
}
