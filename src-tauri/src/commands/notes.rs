use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteListItem {
    pub id: String,
    pub title: String,
    pub summary: String,
}

#[tauri::command]
pub fn list_notes_command() -> Vec<NoteListItem> {
    Vec::new()
}
