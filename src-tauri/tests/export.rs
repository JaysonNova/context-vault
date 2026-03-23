use std::path::PathBuf;

use rusqlite::{params, Connection};
use tempfile::tempdir;

use context_vault_lib::db::migrations::run_migrations;
use context_vault_lib::export::export_note_markdown;

fn seed_note_db() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    run_migrations(&connection).unwrap();

    connection
        .execute(
            "INSERT INTO notes (
                id, title, summary, body_md, status, created_at, updated_at, model_provider, model_name, prompt_version
            ) VALUES (?1, ?2, ?3, ?4, 'ready', ?5, ?5, 'stub', 'stub', 'technical_note_v1')",
            params![
                "note_1",
                "gRPC 排障笔记",
                "连接池上限问题",
                "# gRPC 排障笔记\n\n连接池上限问题",
                100_i64
            ],
        )
        .unwrap();

    connection
}

#[test]
fn export_note_writes_markdown_to_requested_path() {
    let db = seed_note_db();
    let dir = tempdir().unwrap();
    let path = PathBuf::from(dir.path()).join("grpc-note.md");

    export_note_markdown(&db, "note_1", &path).unwrap();

    let body = std::fs::read_to_string(path).unwrap();
    assert!(body.contains("# gRPC 排障笔记"));
}
