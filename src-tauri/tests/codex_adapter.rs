use std::path::PathBuf;
use tempfile::tempdir;

use context_vault_lib::adapters::codex::CodexAdapter;

#[test]
fn codex_adapter_builds_partial_conversation_from_history_and_threads() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sources/codex");
    let parsed = CodexAdapter::parse_fixture_dir(fixture_dir).unwrap();

    assert_eq!(parsed.conversation.source_app, "codex");
    assert_eq!(parsed.conversation.sync_strength, "partial");
    assert!(parsed.messages.iter().any(|message| message.role == "user"));
    assert!(parsed
        .messages
        .iter()
        .any(|message| message.message_type == "assistant"));
    assert!(parsed
        .messages
        .iter()
        .any(|message| message.message_type == "tool_call"));
    assert!(parsed
        .messages
        .iter()
        .any(|message| message.message_type == "tool_result"));
}

#[test]
fn codex_adapter_falls_back_to_user_only_timeline_when_session_file_is_missing() {
    let temp_dir = tempdir().unwrap();
    let fixture_dir = temp_dir.path();

    std::fs::write(
        fixture_dir.join("history.jsonl"),
        "{\"session_id\":\"codex-session-1\",\"ts\":1773026237,\"text\":\"只保留用户输入\"}\n",
    )
    .unwrap();
    std::fs::write(
        fixture_dir.join("thread_rows.json"),
        r#"
        [
          {
            "id": "codex-session-1",
            "title": "缺少 session transcript",
            "cwd": "/tmp/kiko-app",
            "git_branch": "main",
            "git_origin_url": "https://github.com/example/kiko-app.git",
            "updated_at": 1773026257,
            "rollout_path": "missing-session.jsonl"
          }
        ]
        "#,
    )
    .unwrap();

    let parsed = CodexAdapter::parse_fixture_dir(fixture_dir).unwrap();

    assert_eq!(parsed.conversation.sync_strength, "partial");
    assert_eq!(parsed.messages.len(), 1);
    assert!(parsed.messages.iter().all(|message| message.role == "user"));
}
