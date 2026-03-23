use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncImportCount {
    pub source_app: String,
    pub conversations: i64,
    pub messages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncResult {
    pub source_app: String,
    pub status: String,
    pub imported_conversation_count: i64,
    pub imported_message_count: i64,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRunResult {
    pub id: String,
    pub status: String,
    pub source_results: Vec<SourceSyncResult>,
}

pub fn run_sync_with_stub_adapters(
    results: Vec<Result<SyncImportCount, Error>>,
) -> Result<SyncRunResult> {
    let mut source_results = Vec::new();
    let mut has_success = false;
    let mut has_failure = false;

    for (index, result) in results.into_iter().enumerate() {
        match result {
            Ok(count) => {
                has_success = true;
                source_results.push(SourceSyncResult {
                    source_app: count.source_app,
                    status: "success".into(),
                    imported_conversation_count: count.conversations,
                    imported_message_count: count.messages,
                    error_summary: None,
                });
            }
            Err(error) => {
                has_failure = true;
                source_results.push(SourceSyncResult {
                    source_app: format!("adapter_{index}"),
                    status: "failed".into(),
                    imported_conversation_count: 0,
                    imported_message_count: 0,
                    error_summary: Some(error.to_string()),
                });
            }
        }
    }

    let status = match (has_success, has_failure) {
        (true, true) => "partial_success",
        (true, false) => "success",
        (false, true) => "failed",
        (false, false) => "success",
    };

    Ok(SyncRunResult {
        id: Uuid::new_v4().to_string(),
        status: status.into(),
        source_results,
    })
}
