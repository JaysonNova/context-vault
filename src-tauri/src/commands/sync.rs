use serde::{Deserialize, Serialize};

use crate::db::connection::open_default_db;
use crate::sync::runner::{run_sync, SyncRunResult};

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
pub fn list_sync_runs_command() -> Result<Vec<SyncRunListItem>, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT source_app, status, imported_conversation_count, error_summary
             FROM sync_runs
             ORDER BY started_at DESC",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(SyncRunListItem {
                source_app: row.get(0)?,
                status: row.get(1)?,
                imported_conversation_count: row.get(2)?,
                error_summary: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|error| error.to_string())?);
    }

    Ok(items)
}

#[tauri::command]
pub fn run_sync_command(_request: RunSyncRequest) -> Result<SyncRunResult, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    run_sync(&connection).map_err(|error| error.to_string())
}
