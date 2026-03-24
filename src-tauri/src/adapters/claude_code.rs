use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use glob::glob;
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::sync::normalize::{ConversationImport, ImportedConversation, MessageImport};

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn parse_fixture<P: AsRef<Path>>(path: P) -> Result<ImportedConversation> {
        Self::parse_file(path)
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ImportedConversation> {
        let contents = fs::read_to_string(path)?;
        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut git_branch: Option<String> = None;
        let mut first_user_text: Option<String> = None;
        let mut created_at: Option<i64> = None;
        let mut updated_at: Option<i64> = None;
        let mut messages = Vec::new();

        for line in contents.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line)?;

            let entry_type = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();

            if entry_type == "file-history-snapshot" {
                continue;
            }

            let Some(timestamp) = value.get("timestamp").and_then(Value::as_str) else {
                continue;
            };

            let ts_millis = parse_rfc3339_millis(timestamp)?;
            created_at = Some(created_at.map_or(ts_millis, |current| current.min(ts_millis)));
            updated_at = Some(updated_at.map_or(ts_millis, |current| current.max(ts_millis)));

            if session_id.is_none() {
                session_id = value
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }

            if cwd.is_none() {
                cwd = value.get("cwd").and_then(Value::as_str).map(ToOwned::to_owned);
            }

            if git_branch.is_none() {
                git_branch = value
                    .get("gitBranch")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }

            if let Some(message) = normalize_message(&value, ts_millis)? {
                if first_user_text.is_none()
                    && message.role == "user"
                    && message.message_type == "user"
                    && !message.content_text.is_empty()
                {
                    first_user_text = Some(message.content_text.clone());
                }

                messages.push(message);
            }
        }

        let session_id = session_id.ok_or_else(|| anyhow!("missing session id in fixture"))?;
        let created_at = created_at.ok_or_else(|| anyhow!("missing created_at in fixture"))?;
        let updated_at = updated_at.ok_or_else(|| anyhow!("missing updated_at in fixture"))?;

        let title = first_user_text
            .as_deref()
            .map(truncate_title)
            .unwrap_or("Untitled Claude Code Conversation")
            .to_string();

        let conversation = ConversationImport {
            source_app: "claude_code".into(),
            source_conversation_id: session_id.clone(),
            title,
            subtitle: cwd.clone(),
            created_at,
            updated_at,
            sync_strength: "full".into(),
            raw_metadata_json: json!({
                "session_id": session_id,
                "cwd": cwd,
                "git_branch": git_branch
            })
            .to_string(),
        };

        Ok(ImportedConversation {
            conversation,
            messages,
        })
    }

    pub fn import_default_sources() -> Result<Vec<ImportedConversation>> {
        let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
        let pattern = format!("{home}/.claude/projects/**/*.jsonl");
        let mut conversations = Vec::new();

        for path in glob(&pattern)? {
            conversations.push(Self::parse_file(path?)?);
        }

        Ok(conversations)
    }
}

fn normalize_message(value: &Value, ts_millis: i64) -> Result<Option<MessageImport>> {
    let entry_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let uuid = value
        .get("uuid")
        .and_then(Value::as_str)
        .unwrap_or("unknown-message")
        .to_string();

    match entry_type {
        "user" => normalize_user_message(value, uuid, ts_millis),
        "assistant" => normalize_assistant_message(value, uuid, ts_millis),
        _ => Ok(None),
    }
}

fn normalize_user_message(
    value: &Value,
    uuid: String,
    ts_millis: i64,
) -> Result<Option<MessageImport>> {
    let Some(message) = value.get("message") else {
        return Ok(None);
    };

    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return Ok(Some(MessageImport {
            source_message_id: uuid,
            role: "user".into(),
            message_type: "user".into(),
            content_text: text.to_string(),
            tool_name: None,
            created_at: ts_millis,
            raw_payload_json: message.to_string(),
        }));
    }

    if let Some(items) = message.get("content").and_then(Value::as_array) {
        for item in items {
            if item.get("type").and_then(Value::as_str) == Some("tool_result") {
                let content_text = item
                    .get("content")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        value.get("toolUseResult")
                            .and_then(|tool| tool.get("stdout"))
                            .and_then(Value::as_str)
                    })
                    .unwrap_or_default()
                    .to_string();

                return Ok(Some(MessageImport {
                    source_message_id: uuid,
                    role: "tool".into(),
                    message_type: "tool_result".into(),
                    content_text,
                    tool_name: None,
                    created_at: ts_millis,
                    raw_payload_json: item.to_string(),
                }));
            }
        }
    }

    Ok(None)
}

fn normalize_assistant_message(
    value: &Value,
    uuid: String,
    ts_millis: i64,
) -> Result<Option<MessageImport>> {
    let Some(message) = value.get("message") else {
        return Ok(None);
    };

    let Some(items) = message.get("content").and_then(Value::as_array) else {
        return Ok(None);
    };

    for item in items {
        match item.get("type").and_then(Value::as_str).unwrap_or_default() {
            "text" => {
                return Ok(Some(MessageImport {
                    source_message_id: uuid.clone(),
                    role: "assistant".into(),
                    message_type: "assistant".into(),
                    content_text: item
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    tool_name: None,
                    created_at: ts_millis,
                    raw_payload_json: item.to_string(),
                }));
            }
            "tool_use" => {
                let tool_name = item
                    .get("name")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                let content_text = item
                    .get("input")
                    .map(format_tool_input)
                    .filter(|value| !value.is_empty())
                    .or_else(|| tool_name.clone())
                    .unwrap_or_default();

                return Ok(Some(MessageImport {
                    source_message_id: uuid.clone(),
                    role: "assistant".into(),
                    message_type: "tool_call".into(),
                    content_text,
                    tool_name,
                    created_at: ts_millis,
                    raw_payload_json: item.to_string(),
                }));
            }
            _ => {}
        }
    }

    Ok(None)
}

fn format_tool_input(value: &Value) -> String {
    if let Some(command) = value.get("command").and_then(Value::as_str) {
        return command.to_string();
    }

    if let Some(text) = value.as_str() {
        return text.to_string();
    }

    serde_json::to_string_pretty(value).unwrap_or_default()
}

fn parse_rfc3339_millis(value: &str) -> Result<i64> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339)?;
    Ok(parsed.unix_timestamp_nanos() as i64 / 1_000_000)
}

fn truncate_title(input: &str) -> &str {
    const MAX_CHARS: usize = 48;
    if input.chars().count() <= MAX_CHARS {
        return input;
    }

    let end = input
        .char_indices()
        .nth(MAX_CHARS)
        .map(|(index, _)| index)
        .unwrap_or(input.len());

    &input[..end]
}
