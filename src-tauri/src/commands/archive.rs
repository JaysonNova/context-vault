use rusqlite::{named_params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::connection::open_default_db;
use crate::db::search::build_like_pattern;
use crate::error::{AppError, AppResult};
use crate::models::{
    ArchiveFacets, ArchiveSourceCount, ConversationDetail, ConversationDetailMessage,
    ConversationListItem,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ArchiveQuery {
    pub text: Option<String>,
    pub source_app: Option<String>,
    pub sync_strength: Option<String>,
    pub workspace_id: Option<String>,
    pub note_state: Option<String>,
}

pub fn list_conversations(
    connection: &Connection,
    query: ArchiveQuery,
) -> AppResult<Vec<ConversationListItem>> {
    let text = build_like_pattern(query.text.clone());
    let mut statement = connection.prepare(
        "
        SELECT
          conversations.id,
          conversations.title,
          conversations.source_app,
          conversations.sync_strength,
          workspaces.display_name,
          conversations.updated_at,
          (
            SELECT COUNT(*)
            FROM messages
            WHERE messages.conversation_id = conversations.id
          ) AS message_count,
          COALESCE((
            SELECT content_text
            FROM messages
            WHERE messages.conversation_id = conversations.id
              AND messages.message_type IN ('assistant', 'user')
            ORDER BY created_at DESC
            LIMIT 1
          ), (
            SELECT content_text
            FROM messages
            WHERE messages.conversation_id = conversations.id
            ORDER BY created_at DESC
            LIMIT 1
          ), conversations.subtitle, '') AS preview_text,
          COUNT(note_sources.note_id) AS note_count
        FROM conversations
        LEFT JOIN workspaces
          ON workspaces.id = conversations.workspace_id
        LEFT JOIN note_sources
          ON note_sources.conversation_id = conversations.id
        WHERE conversations.status <> 'deleted'
          AND (
            :text IS NULL
            OR conversations.title LIKE :text
            OR workspaces.display_name LIKE :text
            OR EXISTS(
                SELECT 1
                FROM messages
                WHERE messages.conversation_id = conversations.id
                  AND messages.content_text LIKE :text
            )
          )
          AND (:source_app IS NULL OR conversations.source_app = :source_app)
          AND (:sync_strength IS NULL OR conversations.sync_strength = :sync_strength)
          AND (:workspace_id IS NULL OR conversations.workspace_id = :workspace_id)
          AND (
            :note_state IS NULL
            OR (:note_state = 'with_notes' AND EXISTS(
              SELECT 1 FROM note_sources WHERE note_sources.conversation_id = conversations.id
            ))
            OR (:note_state = 'without_notes' AND NOT EXISTS(
              SELECT 1 FROM note_sources WHERE note_sources.conversation_id = conversations.id
            ))
          )
        GROUP BY
          conversations.id,
          conversations.title,
          conversations.source_app,
          conversations.sync_strength,
          workspaces.display_name,
          conversations.updated_at
        ORDER BY conversations.updated_at DESC
        ",
    )?;

    let rows = statement.query_map(
        named_params! {
            ":text": text.as_deref(),
            ":source_app": query.source_app.as_deref(),
            ":sync_strength": query.sync_strength.as_deref(),
            ":workspace_id": query.workspace_id.as_deref(),
            ":note_state": query.note_state.as_deref(),
        },
        |row| {
            Ok(ConversationListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                source_app: row.get(2)?,
                sync_strength: row.get(3)?,
                workspace_name: row.get(4)?,
                updated_at: row.get(5)?,
                message_count: row.get(6)?,
                preview_text: row.get(7)?,
                note_count: row.get(8)?,
            })
        },
    )?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

pub fn get_archive_facets(connection: &Connection) -> AppResult<ArchiveFacets> {
    let total_count = connection.query_row(
        "SELECT COUNT(*) FROM conversations WHERE status <> 'deleted'",
        [],
        |row| row.get::<_, i64>(0),
    )?;

    let mut statement = connection.prepare(
        "
        SELECT source_app, COUNT(*)
        FROM conversations
        WHERE status <> 'deleted'
        GROUP BY source_app
        ORDER BY source_app ASC
        ",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(ArchiveSourceCount {
            source_app: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let mut source_counts = Vec::new();
    for row in rows {
        source_counts.push(row?);
    }

    Ok(ArchiveFacets {
        total_count,
        source_counts,
    })
}

pub fn get_conversation_detail(
    connection: &Connection,
    conversation_id: &str,
) -> AppResult<Option<ConversationDetail>> {
    let detail = connection
        .query_row(
            "
            SELECT
              conversations.id,
              conversations.title,
              conversations.source_app,
              conversations.source_conversation_id,
              conversations.sync_strength,
              workspaces.display_name,
              conversations.updated_at,
              COUNT(note_sources.note_id) AS note_count,
              COALESCE((
                SELECT content_text
                FROM messages
                WHERE messages.conversation_id = conversations.id
                  AND messages.message_type IN ('assistant', 'user')
                ORDER BY created_at DESC
                LIMIT 1
              ), (
                SELECT content_text
                FROM messages
                WHERE messages.conversation_id = conversations.id
                ORDER BY created_at DESC
                LIMIT 1
              ), conversations.subtitle, '') AS preview_text,
              conversations.raw_metadata_json
            FROM conversations
            LEFT JOIN workspaces
              ON workspaces.id = conversations.workspace_id
            LEFT JOIN note_sources
              ON note_sources.conversation_id = conversations.id
            WHERE conversations.id = ?1
              AND conversations.status <> 'deleted'
            GROUP BY
              conversations.id,
              conversations.title,
              conversations.source_app,
              conversations.source_conversation_id,
              conversations.sync_strength,
              workspaces.display_name,
              conversations.updated_at,
              conversations.raw_metadata_json
            ",
            [conversation_id],
            |row| {
                let source_app: String = row.get(2)?;
                let source_conversation_id: String = row.get(3)?;
                let raw_metadata_json: String = row.get(9)?;

                Ok(ConversationDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    source_app: source_app.clone(),
                    sync_strength: row.get(4)?,
                    workspace_name: row.get(5)?,
                    updated_at: row.get(6)?,
                    note_count: row.get(7)?,
                    preview_text: row.get(8)?,
                    raw_metadata_json: raw_metadata_json.clone(),
                    resume_command: build_resume_command(
                        &source_app,
                        &source_conversation_id,
                        &raw_metadata_json,
                    ),
                    messages: Vec::new(),
                })
            },
        )
        .ok();

    let Some(mut detail) = detail else {
        return Ok(None);
    };

    let mut statement = connection.prepare(
        "
        SELECT id, role, message_type, content_text, created_at
        FROM messages
        WHERE conversation_id = ?1
        ORDER BY created_at ASC
        ",
    )?;
    let rows = statement.query_map([conversation_id], |row| {
        Ok(ConversationDetailMessage {
            id: row.get(0)?,
            role: row.get(1)?,
            message_type: row.get(2)?,
            content_text: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    detail.messages = messages;

    Ok(Some(detail))
}

pub fn soft_delete_conversation(connection: &Connection, conversation_id: &str) -> AppResult<()> {
    let source_app = connection
        .query_row(
            "SELECT source_app
             FROM conversations
             WHERE id = ?1
               AND status <> 'deleted'",
            [conversation_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    let Some(source_app) = source_app else {
        return Err(AppError::Message("conversation not found".into()));
    };

    if source_app != "codex" && source_app != "claude_code" {
        return Err(AppError::Message(
            "only codex and claude_code conversations can be deleted".into(),
        ));
    }

    let updated = connection.execute(
        "UPDATE conversations
         SET status = 'deleted'
         WHERE id = ?1",
        [conversation_id],
    )?;

    if updated == 0 {
        return Err(AppError::Message("conversation not found".into()));
    }

    Ok(())
}

fn build_resume_command(
    source_app: &str,
    source_conversation_id: &str,
    raw_metadata_json: &str,
) -> Option<String> {
    let session_id = source_conversation_id.trim();
    if session_id.is_empty() {
        return None;
    }

    let cwd = serde_json::from_str::<Value>(raw_metadata_json)
        .ok()
        .and_then(|metadata| metadata.get("cwd").and_then(Value::as_str).map(str::to_owned))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());

    match source_app {
        "codex" => Some(match cwd {
            Some(cwd) => format!("codex resume -C {} {}", shell_escape(&cwd), session_id),
            None => format!("codex resume {}", session_id),
        }),
        "claude_code" => Some(match cwd {
            Some(cwd) => format!("cd {} && claude --resume {}", shell_escape(&cwd), session_id),
            None => format!("claude --resume {}", session_id),
        }),
        _ => None,
    }
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[tauri::command]
pub fn list_conversations_command(
    query: Option<ArchiveQuery>,
) -> Result<Vec<ConversationListItem>, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    list_conversations(&connection, query.unwrap_or_default()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_archive_facets_command() -> Result<ArchiveFacets, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    get_archive_facets(&connection).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_conversation_detail_command(
    conversation_id: String,
) -> Result<Option<ConversationDetail>, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    get_conversation_detail(&connection, &conversation_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn soft_delete_conversation_command(conversation_id: String) -> Result<(), String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    soft_delete_conversation(&connection, &conversation_id).map_err(|error| error.to_string())
}
