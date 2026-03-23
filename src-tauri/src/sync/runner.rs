use anyhow::{Error, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapters::claude_code::ClaudeCodeAdapter;
use crate::adapters::codex::CodexAdapter;
use crate::adapters::cursor::CursorAdapter;
use crate::sync::normalize::ImportedConversation;

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

pub fn run_sync(connection: &Connection) -> Result<SyncRunResult> {
    let adapter_results: Vec<(String, Result<Vec<ImportedConversation>, Error>)> = vec![
        (
            "claude_code".into(),
            ClaudeCodeAdapter::import_default_sources().map_err(Error::from),
        ),
        (
            "codex".into(),
            CodexAdapter::import_default_sources().map_err(Error::from),
        ),
        (
            "cursor".into(),
            CursorAdapter::import_default_sources().map_err(Error::from),
        ),
    ];

    let mut source_results = Vec::new();
    let mut has_success = false;
    let mut has_failure = false;

    for (source_app, result) in adapter_results {
        match result {
            Ok(imports) => {
                has_success = true;
                let mut imported_messages = 0_i64;
                for import in &imports {
                    imported_messages += import.messages.len() as i64;
                    persist_imported_conversation(connection, import)?;
                }
                persist_sync_run(
                    connection,
                    &source_app,
                    "success",
                    imports.len() as i64,
                    imported_messages,
                    None,
                )?;
                source_results.push(SourceSyncResult {
                    source_app,
                    status: "success".into(),
                    imported_conversation_count: imports.len() as i64,
                    imported_message_count: imported_messages,
                    error_summary: None,
                });
            }
            Err(error) => {
                has_failure = true;
                persist_sync_run(
                    connection,
                    &source_app,
                    "failed",
                    0,
                    0,
                    Some(error.to_string().as_str()),
                )?;
                source_results.push(SourceSyncResult {
                    source_app,
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

fn persist_imported_conversation(
    connection: &Connection,
    imported: &ImportedConversation,
) -> Result<()> {
    let conversation_id = format!(
        "{}:{}",
        imported.conversation.source_app, imported.conversation.source_conversation_id
    );
    let workspace_id = imported
        .conversation
        .subtitle
        .as_ref()
        .filter(|value| value.starts_with('/'))
        .map(|value| value.to_string());

    if let Some(workspace_path) = &workspace_id {
        let display_name = workspace_path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(workspace_path);
        connection.execute(
            "INSERT INTO workspaces (id, normalized_path, display_name, git_branch, git_origin_url, last_seen_at)
             VALUES (?1, ?2, ?3, NULL, NULL, ?4)
             ON CONFLICT(id) DO UPDATE SET display_name = excluded.display_name, last_seen_at = excluded.last_seen_at",
            params![
                workspace_path,
                workspace_path,
                display_name,
                imported.conversation.updated_at
            ],
        )?;
    }

    connection.execute(
        "INSERT INTO conversations (
            id, source_app, source_conversation_id, workspace_id, title, subtitle,
            created_at, updated_at, sync_strength, status, raw_metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'ready', ?10)
        ON CONFLICT(source_app, source_conversation_id) DO UPDATE SET
            workspace_id = excluded.workspace_id,
            title = excluded.title,
            subtitle = excluded.subtitle,
            created_at = excluded.created_at,
            updated_at = excluded.updated_at,
            sync_strength = excluded.sync_strength,
            raw_metadata_json = excluded.raw_metadata_json",
        params![
            conversation_id,
            imported.conversation.source_app,
            imported.conversation.source_conversation_id,
            workspace_id,
            imported.conversation.title,
            imported.conversation.subtitle,
            imported.conversation.created_at,
            imported.conversation.updated_at,
            imported.conversation.sync_strength,
            imported.conversation.raw_metadata_json
        ],
    )?;

    connection.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;

    for (index, message) in imported.messages.iter().enumerate() {
        connection.execute(
            "INSERT INTO messages (
                id, conversation_id, source_message_id, role, message_type, content_text,
                tool_name, token_count, created_at, raw_payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9)",
            params![
                format!("{conversation_id}:{index}"),
                conversation_id,
                message.source_message_id,
                message.role,
                message.message_type,
                message.content_text,
                message.tool_name,
                message.created_at,
                message.raw_payload_json
            ],
        )?;
    }

    Ok(())
}

fn persist_sync_run(
    connection: &Connection,
    source_app: &str,
    status: &str,
    conversations: i64,
    messages: i64,
    error_summary: Option<&str>,
) -> Result<()> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp_nanos() as i64 / 1_000_000;
    connection.execute(
        "INSERT INTO sync_runs (
            id, started_at, finished_at, source_app, status,
            imported_conversation_count, imported_message_count, warning_count, error_summary
        ) VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
        params![
            Uuid::new_v4().to_string(),
            now,
            source_app,
            status,
            conversations,
            messages,
            error_summary
        ],
    )?;
    Ok(())
}
