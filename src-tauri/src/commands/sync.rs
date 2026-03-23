use serde::{Deserialize, Serialize};

use crate::sync::runner::{run_sync_with_stub_adapters, SyncImportCount, SyncRunResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRunListItem {
    pub source_app: String,
    pub status: String,
    pub imported_conversation_count: i64,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSyncRequest {
    pub trigger: String,
}

#[tauri::command]
pub fn list_sync_runs_command() -> Vec<SyncRunListItem> {
    Vec::new()
}

#[tauri::command]
pub fn run_sync_command(_request: RunSyncRequest) -> SyncRunResult {
    run_sync_with_stub_adapters(vec![Ok(SyncImportCount {
        source_app: "startup".into(),
        conversations: 0,
        messages: 0,
    })])
    .expect("stub sync runner should not fail")
}
