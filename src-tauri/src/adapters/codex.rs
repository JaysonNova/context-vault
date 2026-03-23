use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::sync::normalize::{ConversationImport, ImportedConversation, MessageImport};

pub struct CodexAdapter;

#[derive(Debug, Deserialize, Serialize)]
struct HistoryRow {
    session_id: String,
    ts: i64,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ThreadRow {
    id: String,
    title: Option<String>,
    cwd: Option<String>,
    git_branch: Option<String>,
    git_origin_url: Option<String>,
    updated_at: Option<i64>,
}

impl CodexAdapter {
    pub fn parse_fixture_dir<P: AsRef<Path>>(fixture_dir: P) -> Result<ImportedConversation> {
        let fixture_dir = fixture_dir.as_ref();
        let history_path = fixture_dir.join("history.jsonl");
        let thread_rows_path = fixture_dir.join("thread_rows.json");

        let history_rows: Vec<HistoryRow> = fs::read_to_string(history_path)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        let thread_rows: Vec<ThreadRow> = serde_json::from_str(&fs::read_to_string(thread_rows_path)?)?;

        let selected_thread = thread_rows
            .first()
            .ok_or_else(|| anyhow!("missing thread rows in fixture"))?;

        let messages: Vec<MessageImport> = history_rows
            .iter()
            .filter(|row| row.session_id == selected_thread.id)
            .map(|row| MessageImport {
                source_message_id: format!("{}:{}", row.session_id, row.ts),
                role: "user".into(),
                message_type: "user".into(),
                content_text: row.text.clone(),
                tool_name: None,
                created_at: row.ts * 1000,
                raw_payload_json: serde_json::to_string(row).unwrap_or_default(),
            })
            .collect();

        let first_message = messages
            .first()
            .ok_or_else(|| anyhow!("missing history rows for selected thread"))?;

        let metadata = HashMap::from([
            ("cwd", selected_thread.cwd.clone()),
            ("git_branch", selected_thread.git_branch.clone()),
            ("git_origin_url", selected_thread.git_origin_url.clone()),
        ]);

        let conversation = ConversationImport {
            source_app: "codex".into(),
            source_conversation_id: selected_thread.id.clone(),
            title: selected_thread
                .title
                .clone()
                .unwrap_or_else(|| first_message.content_text.clone()),
            subtitle: selected_thread.cwd.clone(),
            created_at: first_message.created_at,
            updated_at: selected_thread
                .updated_at
                .map(|value| value * 1000)
                .unwrap_or(first_message.created_at),
            sync_strength: "partial".into(),
            raw_metadata_json: serde_json::to_string(&metadata)?,
        };

        Ok(ImportedConversation {
            conversation,
            messages,
        })
    }
}
