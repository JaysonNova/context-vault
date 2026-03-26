use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationListItem {
    pub id: String,
    pub title: String,
    pub source_app: String,
    pub sync_strength: String,
    pub workspace_name: Option<String>,
    pub updated_at: i64,
    pub message_count: i64,
    pub preview_text: String,
    pub note_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveSourceCount {
    pub source_app: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveFacets {
    pub total_count: i64,
    pub source_counts: Vec<ArchiveSourceCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationDetailMessage {
    pub id: String,
    pub role: String,
    pub message_type: String,
    pub content_text: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationDetail {
    pub id: String,
    pub title: String,
    pub source_app: String,
    pub sync_strength: String,
    pub workspace_name: Option<String>,
    pub updated_at: i64,
    pub note_count: i64,
    pub preview_text: String,
    pub raw_metadata_json: String,
    pub resume_command: Option<String>,
    pub messages: Vec<ConversationDetailMessage>,
}
