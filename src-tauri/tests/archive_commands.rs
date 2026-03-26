use rusqlite::{params, Connection};

use context_vault_lib::commands::archive::{
    get_archive_facets, get_conversation_detail, list_conversations, soft_delete_conversation,
    ArchiveQuery,
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
            ) VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, 'ready', ?8)",
            params![
                "conv_1",
                "codex",
                "source_1",
                "Older conversation",
                100_i64,
                100_i64,
                "partial",
                r#"{"cwd":"/tmp/codex project"}"#
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
    assert_eq!(detail.resume_command.as_deref(), Some("claude --resume source_2"));
}

#[test]
fn conversation_detail_returns_codex_resume_command_with_cwd_when_available() {
    let db = seed_archive_db();
    let detail = get_conversation_detail(&db, "conv_1").unwrap().unwrap();

    assert_eq!(
        detail.resume_command.as_deref(),
        Some("codex resume -C '/tmp/codex project' source_1")
    );
}

#[test]
fn conversation_detail_returns_short_codex_resume_command_when_cwd_is_missing() {
    let db = seed_archive_db();
    db.execute(
        "UPDATE conversations SET raw_metadata_json = '{}'
         WHERE id = 'conv_1'",
        [],
    )
    .unwrap();

    let detail = get_conversation_detail(&db, "conv_1").unwrap().unwrap();

    assert_eq!(detail.resume_command.as_deref(), Some("codex resume source_1"));
}

#[test]
fn conversation_detail_returns_claude_resume_command_with_cwd_when_available() {
    let db = seed_archive_db();
    db.execute(
        "UPDATE conversations
         SET raw_metadata_json = ?1
         WHERE id = 'conv_2'",
        [r#"{"cwd":"/tmp/claude project"}"#],
    )
    .unwrap();

    let detail = get_conversation_detail(&db, "conv_2").unwrap().unwrap();

    assert_eq!(
        detail.resume_command.as_deref(),
        Some("cd '/tmp/claude project' && claude --resume source_2")
    );
}

#[test]
fn conversation_detail_returns_short_claude_resume_command_when_cwd_is_missing() {
    let db = seed_archive_db();

    let detail = get_conversation_detail(&db, "conv_2").unwrap().unwrap();

    assert_eq!(detail.resume_command.as_deref(), Some("claude --resume source_2"));
}

#[test]
fn conversation_detail_omits_resume_command_when_session_id_is_blank() {
    let db = seed_archive_db();
    db.execute(
        "UPDATE conversations
         SET source_conversation_id = '   '
         WHERE id = 'conv_2'",
        [],
    )
    .unwrap();

    let detail = get_conversation_detail(&db, "conv_2").unwrap().unwrap();

    assert_eq!(detail.resume_command, None);
}

#[test]
fn soft_delete_conversation_hides_deleted_records_from_archive_queries() {
    let db = seed_archive_db();

    soft_delete_conversation(&db, "conv_1").unwrap();

    let rows = list_conversations(&db, ArchiveQuery::default()).unwrap();
    let facets = get_archive_facets(&db).unwrap();
    let detail = get_conversation_detail(&db, "conv_1").unwrap();
    let status: String = db
        .query_row(
            "SELECT status FROM conversations WHERE id = 'conv_1'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "deleted");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "conv_2");
    assert_eq!(facets.total_count, 1);
    assert_eq!(facets.source_counts.len(), 1);
    assert_eq!(facets.source_counts[0].source_app, "claude_code");
    assert!(detail.is_none());
}

#[test]
fn soft_delete_conversation_rejects_non_codex_and_non_claude_sources() {
    let db = seed_archive_db();
    db.execute(
        "INSERT INTO conversations (
            id, source_app, source_conversation_id, workspace_id, title, subtitle,
            created_at, updated_at, sync_strength, status, raw_metadata_json
        ) VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, 'ready', '{}')",
        params![
            "conv_3",
            "cursor",
            "source_3",
            "Cursor Conversation",
            300_i64,
            300_i64,
            "metadata_only"
        ],
    )
    .unwrap();

    let error = soft_delete_conversation(&db, "conv_3").unwrap_err();
    let status: String = db
        .query_row(
            "SELECT status FROM conversations WHERE id = 'conv_3'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        error.to_string(),
        "only codex and claude_code conversations can be deleted"
    );
    assert_eq!(status, "ready");
}

#[test]
fn list_conversations_falls_back_to_subtitle_when_metadata_only_has_no_messages() {
    let db = seed_archive_db();
    db.execute(
        "INSERT INTO conversations (
            id, source_app, source_conversation_id, workspace_id, title, subtitle,
            created_at, updated_at, sync_strength, status, raw_metadata_json
        ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, 'ready', ?9)",
        params![
            "conv_3",
            "cursor",
            "cursor-source-1",
            "Untitled Cursor Composer",
            "Edited src/app.ts, docs/spec.md",
            150_i64,
            150_i64,
            "metadata_only",
            r#"{"workspace_path":"/tmp/cursor-workspace"}"#
        ],
    )
    .unwrap();

    let rows = list_conversations(
        &db,
        ArchiveQuery {
            source_app: Some("cursor".into()),
            ..ArchiveQuery::default()
        },
    )
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].preview_text, "Edited src/app.ts, docs/spec.md");
}

#[test]
fn conversation_detail_falls_back_to_subtitle_when_metadata_only_has_no_messages() {
    let db = seed_archive_db();
    db.execute(
        "INSERT INTO conversations (
            id, source_app, source_conversation_id, workspace_id, title, subtitle,
            created_at, updated_at, sync_strength, status, raw_metadata_json
        ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, 'ready', ?9)",
        params![
            "conv_3",
            "cursor",
            "cursor-source-1",
            "Untitled Cursor Composer",
            "Edited src/app.ts, docs/spec.md",
            150_i64,
            150_i64,
            "metadata_only",
            r#"{"workspace_path":"/tmp/cursor-workspace"}"#
        ],
    )
    .unwrap();

    let detail = get_conversation_detail(&db, "conv_3").unwrap().unwrap();

    assert_eq!(detail.preview_text, "Edited src/app.ts, docs/spec.md");
    assert!(detail.messages.is_empty());
}
