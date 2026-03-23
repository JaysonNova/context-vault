use std::path::PathBuf;

use context_vault_lib::adapters::cursor::CursorAdapter;

#[test]
fn cursor_adapter_imports_metadata_only_conversations() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/sources/cursor");
    let parsed = CursorAdapter::parse_fixture_dir(fixture_dir).unwrap();

    assert_eq!(parsed.conversation.source_app, "cursor");
    assert_eq!(parsed.conversation.sync_strength, "metadata_only");
    assert!(parsed.messages.is_empty() || parsed.messages.iter().all(|message| message.message_type == "metadata"));
}
