use rusqlite::{params, Connection};

use context_vault_lib::db::migrations::run_migrations;
use context_vault_lib::notes::generate_note;
use context_vault_lib::notes::provider::StubProvider;

fn seed_conversation_for_note() -> Connection {
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
                "claude-source-1",
                "gRPC 排障会话",
                100_i64,
                200_i64,
                "full"
            ],
        )
        .unwrap();

    connection
        .execute(
            "INSERT INTO messages (
                id, conversation_id, source_message_id, role, message_type, content_text,
                tool_name, token_count, created_at, raw_payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, '{}')",
            params![
                "msg_1",
                "conv_1",
                "msg-source-1",
                "user",
                "user",
                "排查 gRPC 连接池耗尽",
                100_i64
            ],
        )
        .unwrap();

    connection
}

#[test]
fn generate_note_persists_summary_body_and_prompt_version() {
    let db = seed_conversation_for_note();
    let provider = StubProvider::success("# Note\n\nSummary");

    let note = generate_note(&db, provider, "conv_1").unwrap();

    assert_eq!(note.prompt_version, "technical_note_v1");
    assert!(note.body_md.contains("# Note"));
}
