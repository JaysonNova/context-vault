use rusqlite::{params, Connection};

use context_vault_lib::commands::archive::{
    get_archive_facets, get_conversation_detail, list_conversations, ArchiveQuery,
};
use context_vault_lib::db::migrations::run_migrations;

fn seed_archive_db() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    run_migrations(&connection).unwrap();

    connection
        .execute(
            "INSERT INTO workspaces (id, normalized_path, display_name, git_branch, git_origin_url, last_seen_at)
             VALUES (?1, ?2, ?3, NULL, NULL, ?4)",
            params!["workspace_1", "/tmp/kiko-app", "kiko-app", 200_i64],
        )
        .unwrap();

    connection
        .execute(
            "INSERT INTO conversations (
                id, source_app, source_conversation_id, workspace_id, title, subtitle,
                created_at, updated_at, sync_strength, status, raw_metadata_json
            ) VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, 'ready', '{}')",
            params![
                "conv_1",
                "codex",
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
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, 'ready', '{}')",
            params![
                "conv_2",
                "claude_code",
                "source_2",
                "workspace_1",
                "gRPC 连接池耗尽排查",
                200_i64,
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
                "conv_2",
                "source_msg_1",
                "user",
                "user",
                "排查 gRPC RESOURCE_EXHAUSTED 的根因",
                200_i64
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO messages (
                id, conversation_id, source_message_id, role, message_type, content_text,
                tool_name, token_count, created_at, raw_payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, '{}')",
            params![
                "msg_3",
                "conv_2",
                "source_msg_3",
                "assistant",
                "tool_call",
                r#"{"cmd":"rg gRPC"}"#,
                "exec_command",
                202_i64
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO messages (
                id, conversation_id, source_message_id, role, message_type, content_text,
                tool_name, token_count, created_at, raw_payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, '{}')",
            params![
                "msg_4",
                "conv_2",
                "source_msg_4",
                "tool",
                "tool_result",
                "Command output from rg gRPC",
                "exec_command",
                203_i64
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
                "msg_2",
                "conv_2",
                "source_msg_2",
                "assistant",
                "assistant",
                "根因是服务端并发流上限过低。",
                201_i64
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO notes (
                id, title, summary, body_md, status, created_at, updated_at, model_provider, model_name, prompt_version
            ) VALUES (?1, ?2, ?3, ?4, 'ready', ?5, ?5, 'stub', 'stub', 'technical_note_v1')",
            params![
                "note_1",
                "gRPC 排障笔记",
                "连接池上限过低",
                "# gRPC 排障笔记",
                200_i64
            ],
        )
        .unwrap();

    connection
        .execute(
            "INSERT INTO note_sources (note_id, conversation_id) VALUES (?1, ?2)",
            params!["note_1", "conv_2"],
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
    assert_eq!(rows[0].message_count, 4);
    assert_eq!(rows[0].preview_text, "根因是服务端并发流上限过低。");
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

#[test]
fn archive_facets_return_source_counts() {
    let db = seed_archive_db();
    let facets = get_archive_facets(&db).unwrap();

    assert_eq!(facets.total_count, 2);
    assert_eq!(facets.source_counts.len(), 2);
    assert_eq!(facets.source_counts[0].source_app, "claude_code");
}

#[test]
fn conversation_detail_returns_workspace_and_message_timeline() {
    let db = seed_archive_db();
    let detail = get_conversation_detail(&db, "conv_2").unwrap().unwrap();

    assert_eq!(detail.workspace_name.as_deref(), Some("kiko-app"));
    assert_eq!(detail.messages.len(), 4);
    assert_eq!(detail.messages[0].content_text, "排查 gRPC RESOURCE_EXHAUSTED 的根因");
}
