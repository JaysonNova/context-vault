use std::path::PathBuf;

use context_vault_lib::adapters::claude_code::ClaudeCodeAdapter;

#[test]
fn claude_adapter_parses_full_conversation_with_tool_events() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/sources/claude/projects/sample-session.jsonl");
    let result = ClaudeCodeAdapter::parse_fixture(fixture_path).unwrap();

    assert_eq!(result.conversation.source_app, "claude_code");
    assert_eq!(result.conversation.sync_strength, "full");
    assert!(result
        .messages
        .iter()
        .any(|message| message.message_type == "tool_call"));
    assert!(result
        .messages
        .iter()
        .any(|message| message.message_type == "tool_result"));
}
