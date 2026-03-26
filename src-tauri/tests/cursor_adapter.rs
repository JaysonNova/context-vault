use std::path::PathBuf;

use context_vault_lib::adapters::cursor::CursorAdapter;
use serde_json::Value;

#[test]
fn cursor_adapter_recovers_partial_and_metadata_only_conversations() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sources/cursor");
    let parsed = CursorAdapter::parse_fixture_dir(fixture_dir).unwrap();

    assert_eq!(parsed.len(), 3);

    let partial = parsed
        .iter()
        .find(|item| item.conversation.source_conversation_id == "cursor-partial-1")
        .unwrap();
    let partial_metadata: Value =
        serde_json::from_str(&partial.conversation.raw_metadata_json).unwrap();

    assert_eq!(partial.conversation.source_app, "cursor");
    assert_eq!(partial.conversation.sync_strength, "partial");
    assert_eq!(
        partial.conversation.title,
        "Historical records module interface adjustment"
    );
    assert_eq!(
        partial.conversation.subtitle.as_deref(),
        Some("Edited history_repository.dart, history_api_repository.dart")
    );
    assert_eq!(partial.conversation.created_at, partial.messages.first().unwrap().created_at);
    assert_eq!(partial.conversation.updated_at, partial.messages.last().unwrap().created_at);
    assert_eq!(partial.messages.len(), 4);
    assert_eq!(partial.messages[0].role, "user");
    assert_eq!(partial.messages[0].message_type, "user");
    assert!(partial.messages[0].content_text.contains("审视项目"));
    assert_eq!(partial.messages[1].role, "assistant");
    assert_eq!(partial.messages[1].message_type, "assistant");
    assert!(partial.messages[1].content_text.contains("reading the API document"));
    assert_eq!(partial.messages[2].message_type, "tool_call");
    assert_eq!(
        partial.messages[2].tool_name.as_deref(),
        Some("run_terminal_command_v2")
    );
    assert_eq!(partial.messages[2].content_text, "git status 2>&1");
    assert_eq!(partial.messages[3].role, "tool");
    assert_eq!(partial.messages[3].message_type, "tool_result");
    assert!(partial.messages[3].content_text.contains("On branch dev_taofang11"));
    assert!(partial
        .messages
        .iter()
        .all(|message| !message.content_text.contains("review the project and start integrating")));
    assert_eq!(
        partial_metadata.get("workspace_path").and_then(Value::as_str),
        Some("/Users/fangtao/Documents/Codebase/kiko_app")
    );
    assert_eq!(
        partial_metadata.get("git_branch").and_then(Value::as_str),
        Some("dev_taofang11")
    );
    assert_eq!(
        partial_metadata.get("git_remote").and_then(Value::as_str),
        Some("ssh://code.iflytek.com:30004/CBG_AIM/kiko/kiko_app.git")
    );
    assert_eq!(
        partial_metadata
            .get("missing_bubble_count")
            .and_then(Value::as_i64),
        Some(0)
    );
    assert_eq!(
        partial_metadata
            .get("ignored_thinking_block_count")
            .and_then(Value::as_i64),
        Some(1)
    );

    let metadata_fallback = parsed
        .iter()
        .find(|item| item.conversation.source_conversation_id == "cursor-metadata-1")
        .unwrap();
    let metadata_fallback_raw: Value =
        serde_json::from_str(&metadata_fallback.conversation.raw_metadata_json).unwrap();

    assert_eq!(metadata_fallback.conversation.sync_strength, "metadata_only");
    assert_eq!(
        metadata_fallback.conversation.title,
        "Metadata fallback conversation"
    );
    assert!(metadata_fallback.messages.is_empty());
    assert_eq!(
        metadata_fallback_raw
            .get("fallback_reason")
            .and_then(Value::as_str),
        Some("no_recoverable_bubbles")
    );
    assert_eq!(
        metadata_fallback_raw
            .get("missing_bubble_count")
            .and_then(Value::as_i64),
        Some(2)
    );

    let workspace_only = parsed
        .iter()
        .find(|item| item.conversation.source_conversation_id == "cursor-workspace-only-1")
        .unwrap();
    let workspace_only_raw: Value =
        serde_json::from_str(&workspace_only.conversation.raw_metadata_json).unwrap();

    assert_eq!(workspace_only.conversation.sync_strength, "metadata_only");
    assert_eq!(
        workspace_only.conversation.title,
        "Workspace only conversation"
    );
    assert!(workspace_only.messages.is_empty());
    assert_eq!(
        workspace_only_raw
            .get("fallback_reason")
            .and_then(Value::as_str),
        Some("workspace_only")
    );
    assert_eq!(
        workspace_only_raw.get("workspace_path").and_then(Value::as_str),
        Some("/Users/fangtao/Documents/Codebase/context-vault")
    );
}
