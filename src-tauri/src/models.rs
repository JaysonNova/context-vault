use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationListItem {
    pub id: String,
    pub title: String,
    pub source_app: String,
    pub sync_strength: String,
    pub workspace_name: Option<String>,
    pub updated_at: i64,
    pub note_count: i64,
}
