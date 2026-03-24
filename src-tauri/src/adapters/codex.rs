use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use glob::glob;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
    rollout_path: Option<String>,
    updated_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SessionEvent {
    timestamp: String,
    #[serde(rename = "type")]
    event_type: String,
    payload: Value,
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

        let history_messages = build_history_messages(&history_rows, &selected_thread.id);
        let transcript_messages =
            load_rollout_messages_if_available(Some(fixture_dir), selected_thread.rollout_path.as_deref());
        let messages = merge_messages(history_messages.clone(), transcript_messages);

        let first_message = history_messages
            .first()
            .or(messages.first())
            .ok_or_else(|| anyhow!("missing history rows for selected thread"))?;

        let metadata = HashMap::from([
            ("cwd", selected_thread.cwd.clone()),
            ("git_branch", selected_thread.git_branch.clone()),
            ("git_origin_url", selected_thread.git_origin_url.clone()),
            ("rollout_path", selected_thread.rollout_path.clone()),
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
                "SELECT id, title, cwd, git_branch, git_origin_url, rollout_path, updated_at FROM threads",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(ThreadRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    cwd: row.get(2)?,
                    git_branch: row.get(3)?,
                    git_origin_url: row.get(4)?,
                    rollout_path: row.get(5)?,
                    updated_at: row.get(6)?,
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
            let history_messages = build_history_messages(&rows, &session_id);
            let transcript_messages =
                load_rollout_messages_if_available(None, thread.and_then(|value| value.rollout_path.as_deref()));
            let messages = merge_messages(history_messages.clone(), transcript_messages);

            if let Some(first_message) = history_messages.first().or(messages.first()) {
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
                    (
                        "rollout_path",
                        thread.and_then(|value| value.rollout_path.clone()),
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

fn build_history_messages(history_rows: &[HistoryRow], session_id: &str) -> Vec<MessageImport> {
    history_rows
        .iter()
        .filter(|row| row.session_id == session_id)
        .map(|row| MessageImport {
            source_message_id: format!("{}:{}", row.session_id, row.ts),
            role: "user".into(),
            message_type: "user".into(),
            content_text: row.text.clone(),
            tool_name: None,
            created_at: normalize_epoch_millis(row.ts),
            raw_payload_json: serde_json::to_string(row).unwrap_or_default(),
        })
        .collect()
}

fn load_rollout_messages_if_available(
    base_dir: Option<&Path>,
    rollout_path: Option<&str>,
) -> Vec<MessageImport> {
    let Some(rollout_path) = rollout_path else {
        return Vec::new();
    };

    let resolved_path = resolve_rollout_path(base_dir, rollout_path);
    if !resolved_path.exists() {
        return Vec::new();
    }

    load_rollout_messages(&resolved_path).unwrap_or_default()
}

fn resolve_rollout_path(base_dir: Option<&Path>, rollout_path: &str) -> PathBuf {
    let path = PathBuf::from(rollout_path);
    if path.is_absolute() {
        return path;
    }

    base_dir
        .map(|dir| dir.join(path))
        .unwrap_or_else(|| PathBuf::from(rollout_path))
}

fn load_rollout_messages(path: &Path) -> Result<Vec<MessageImport>> {
    let mut messages = Vec::new();

    for line in fs::read_to_string(path)?.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(event) = serde_json::from_str::<SessionEvent>(line) else {
            continue;
        };

        if let Some(message) = normalize_rollout_event(event)? {
            messages.push(message);
        }
    }

    Ok(messages)
}

fn normalize_rollout_event(event: SessionEvent) -> Result<Option<MessageImport>> {
    if event.event_type != "response_item" {
        return Ok(None);
    }

    let payload_type = event
        .payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let created_at = parse_rfc3339_millis(&event.timestamp)?;

    match payload_type {
        "message" => normalize_rollout_assistant_message(&event.payload, created_at),
        "function_call" => Ok(normalize_rollout_function_call(&event.payload, created_at)),
        "function_call_output" => Ok(normalize_rollout_function_result(&event.payload, created_at)),
        _ => Ok(None),
    }
}

fn normalize_rollout_assistant_message(
    payload: &Value,
    created_at: i64,
) -> Result<Option<MessageImport>> {
    if payload.get("role").and_then(Value::as_str) != Some("assistant") {
        return Ok(None);
    }

    let content_items = payload
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let content_text = content_items
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    if content_text.is_empty() {
        return Ok(None);
    }

    Ok(Some(MessageImport {
        source_message_id: format!("assistant:{created_at}"),
        role: "assistant".into(),
        message_type: "assistant".into(),
        content_text,
        tool_name: None,
        created_at,
        raw_payload_json: payload.to_string(),
    }))
}

fn normalize_rollout_function_call(payload: &Value, created_at: i64) -> Option<MessageImport> {
    let tool_name = payload
        .get("name")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let call_id = payload.get("call_id").and_then(Value::as_str);
    let content_text = payload
        .get("arguments")
        .and_then(Value::as_str)
        .map(format_json_like_text)
        .filter(|value| !value.is_empty())
        .or_else(|| tool_name.clone())
        .unwrap_or_default();

    if content_text.is_empty() {
        return None;
    }

    Some(MessageImport {
        source_message_id: call_id
            .map(|value| format!("tool-call:{value}"))
            .unwrap_or_else(|| format!("tool-call:{created_at}")),
        role: "assistant".into(),
        message_type: "tool_call".into(),
        content_text,
        tool_name,
        created_at,
        raw_payload_json: payload.to_string(),
    })
}

fn normalize_rollout_function_result(payload: &Value, created_at: i64) -> Option<MessageImport> {
    let content_text = payload
        .get("output")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    if content_text.is_empty() {
        return None;
    }

    Some(MessageImport {
        source_message_id: payload
            .get("call_id")
            .and_then(Value::as_str)
            .map(|value| format!("tool-result:{value}"))
            .unwrap_or_else(|| format!("tool-result:{created_at}")),
        role: "tool".into(),
        message_type: "tool_result".into(),
        content_text,
        tool_name: None,
        created_at,
        raw_payload_json: payload.to_string(),
    })
}

fn merge_messages(
    mut history_messages: Vec<MessageImport>,
    transcript_messages: Vec<MessageImport>,
) -> Vec<MessageImport> {
    history_messages.extend(transcript_messages);
    history_messages.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.source_message_id.cmp(&right.source_message_id))
    });
    history_messages
}

fn format_json_like_text(value: &str) -> String {
    serde_json::from_str::<Value>(value)
        .ok()
        .and_then(|json| serde_json::to_string_pretty(&json).ok())
        .unwrap_or_else(|| value.to_string())
}

fn normalize_epoch_millis(value: i64) -> i64 {
    if value > 1_000_000_000_000 {
        value
    } else {
        value * 1000
    }
}

fn parse_rfc3339_millis(value: &str) -> Result<i64> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339)?;
    Ok(parsed.unix_timestamp_nanos() as i64 / 1_000_000)
}
