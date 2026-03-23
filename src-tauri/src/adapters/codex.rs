use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use glob::glob;
use rusqlite::Connection;
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

    pub fn import_default_sources() -> Result<Vec<ImportedConversation>> {
        let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
        let history_path = Path::new(&home).join(".codex/history.jsonl");
        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let history_rows: Vec<HistoryRow> = fs::read_to_string(&history_path)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;

        let mut thread_map = HashMap::new();
        let pattern = format!("{home}/.codex/state_*.sqlite");
        for path in glob(&pattern)? {
            let connection = Connection::open(path?)?;
            let mut statement = connection.prepare(
                "SELECT id, title, cwd, git_branch, git_origin_url, updated_at FROM threads",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(ThreadRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    cwd: row.get(2)?,
                    git_branch: row.get(3)?,
                    git_origin_url: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;

            for row in rows {
                let row = row?;
                thread_map.insert(row.id.clone(), row);
            }
        }

        let mut grouped_rows: HashMap<String, Vec<HistoryRow>> = HashMap::new();
        for row in history_rows {
            grouped_rows
                .entry(row.session_id.clone())
                .or_default()
                .push(row);
        }

        let mut conversations = Vec::new();
        for (session_id, mut rows) in grouped_rows {
            rows.sort_by_key(|row| row.ts);
            let thread = thread_map.get(&session_id);
            let messages: Vec<MessageImport> = rows
                .iter()
                .map(|row| MessageImport {
                    source_message_id: format!("{}:{}", row.session_id, row.ts),
                    role: "user".into(),
                    message_type: "user".into(),
                    content_text: row.text.clone(),
                    tool_name: None,
                    created_at: normalize_epoch_millis(row.ts),
                    raw_payload_json: serde_json::to_string(row).unwrap_or_default(),
                })
                .collect();

            if let Some(first_message) = messages.first() {
                let metadata = HashMap::from([
                    (
                        "cwd",
                        thread.and_then(|value| value.cwd.clone()),
                    ),
                    (
                        "git_branch",
                        thread.and_then(|value| value.git_branch.clone()),
                    ),
                    (
                        "git_origin_url",
                        thread.and_then(|value| value.git_origin_url.clone()),
                    ),
                ]);

                conversations.push(ImportedConversation {
                    conversation: ConversationImport {
                        source_app: "codex".into(),
                        source_conversation_id: session_id.clone(),
                        title: thread
                            .and_then(|value| value.title.clone())
                            .unwrap_or_else(|| first_message.content_text.clone()),
                        subtitle: thread.and_then(|value| value.cwd.clone()),
                        created_at: first_message.created_at,
                        updated_at: thread
                            .and_then(|value| value.updated_at)
                            .map(normalize_epoch_millis)
                            .unwrap_or_else(|| {
                                messages
                                    .last()
                                    .map(|message| message.created_at)
                                    .unwrap_or(first_message.created_at)
                            }),
                        sync_strength: "partial".into(),
                        raw_metadata_json: serde_json::to_string(&metadata)?,
                    },
                    messages,
                });
            }
        }

        conversations.sort_by(|left, right| {
            right
                .conversation
                .updated_at
                .cmp(&left.conversation.updated_at)
        });

        Ok(conversations)
    }
}

fn normalize_epoch_millis(value: i64) -> i64 {
    if value > 1_000_000_000_000 {
        value
    } else {
        value * 1000
    }
}
