use std::path::PathBuf;

use context_vault_lib::adapters::codex::CodexAdapter;

#[test]
fn codex_adapter_builds_partial_conversation_from_history_and_threads() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sources/codex");
    let parsed = CodexAdapter::parse_fixture_dir(fixture_dir).unwrap();

    assert_eq!(parsed.conversation.source_app, "codex");
    assert_eq!(parsed.conversation.sync_strength, "partial");
    assert!(parsed.messages.iter().all(|message| message.role == "user"));
}
