use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};

use crate::db::connection::open_default_db;
use crate::db::search::build_like_pattern;
use crate::error::AppResult;
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
            ORDER BY created_at DESC
            LIMIT 1
          ), '') AS preview_text,
          COUNT(note_sources.note_id) AS note_count
        FROM conversations
        LEFT JOIN workspaces
          ON workspaces.id = conversations.workspace_id
        LEFT JOIN note_sources
          ON note_sources.conversation_id = conversations.id
        WHERE (
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
        "SELECT COUNT(*) FROM conversations",
        [],
        |row| row.get::<_, i64>(0),
    )?;

    let mut statement = connection.prepare(
        "
        SELECT source_app, COUNT(*)
        FROM conversations
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
              conversations.sync_strength,
              workspaces.display_name,
              conversations.updated_at,
              COUNT(note_sources.note_id) AS note_count,
              COALESCE((
                SELECT content_text
                FROM messages
                WHERE messages.conversation_id = conversations.id
                ORDER BY created_at DESC
                LIMIT 1
              ), '') AS preview_text,
              conversations.raw_metadata_json
            FROM conversations
            LEFT JOIN workspaces
              ON workspaces.id = conversations.workspace_id
            LEFT JOIN note_sources
              ON note_sources.conversation_id = conversations.id
            WHERE conversations.id = ?1
            GROUP BY
              conversations.id,
              conversations.title,
              conversations.source_app,
              conversations.sync_strength,
              workspaces.display_name,
              conversations.updated_at,
              conversations.raw_metadata_json
            ",
            [conversation_id],
            |row| {
                Ok(ConversationDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    source_app: row.get(2)?,
                    sync_strength: row.get(3)?,
                    workspace_name: row.get(4)?,
                    updated_at: row.get(5)?,
                    note_count: row.get(6)?,
                    preview_text: row.get(7)?,
                    raw_metadata_json: row.get(8)?,
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
