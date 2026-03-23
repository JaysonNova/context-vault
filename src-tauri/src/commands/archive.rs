use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};

use crate::db::connection::open_default_db;
use crate::db::search::build_like_pattern;
use crate::error::AppResult;
use crate::models::ConversationListItem;

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
          COUNT(note_sources.note_id) AS note_count
        FROM conversations
        LEFT JOIN workspaces
          ON workspaces.id = conversations.workspace_id
        LEFT JOIN note_sources
          ON note_sources.conversation_id = conversations.id
        WHERE (:text IS NULL OR conversations.title LIKE :text)
          AND (:source_app IS NULL OR conversations.source_app = :source_app)
          AND (:sync_strength IS NULL OR conversations.sync_strength = :sync_strength)
          AND (:workspace_id IS NULL OR conversations.workspace_id = :workspace_id)
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
        },
        |row| {
            Ok(ConversationListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                source_app: row.get(2)?,
                sync_strength: row.get(3)?,
                workspace_name: row.get(4)?,
                updated_at: row.get(5)?,
                note_count: row.get(6)?,
            })
        },
    )?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

#[tauri::command]
pub fn list_conversations_command() -> Result<Vec<ConversationListItem>, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    list_conversations(&connection, ArchiveQuery::default()).map_err(|error| error.to_string())
}
