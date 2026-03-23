use anyhow::anyhow;

use context_vault_lib::sync::runner::{run_sync_with_stub_adapters, SyncImportCount};

#[test]
fn sync_runner_records_partial_success_per_adapter() {
    let result = run_sync_with_stub_adapters(vec![
        Ok(SyncImportCount {
            source_app: "claude_code".into(),
            conversations: 2,
            messages: 8,
        }),
        Err(anyhow!("cursor db locked")),
    ])
    .unwrap();

    assert_eq!(result.status, "partial_success");
    assert_eq!(result.source_results.len(), 2);
}
