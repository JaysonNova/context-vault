use serde::{Deserialize, Serialize};

use crate::db::connection::open_default_db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteListItem {
    pub id: String,
    pub title: String,
    pub summary: String,
}

#[tauri::command]
pub fn list_notes_command() -> Result<Vec<NoteListItem>, String> {
    let connection = open_default_db().map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare("SELECT id, title, summary FROM notes ORDER BY updated_at DESC")
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(NoteListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|error| error.to_string())?);
    }

    Ok(items)
}
