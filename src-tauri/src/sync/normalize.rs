#[derive(Debug, Clone)]
pub struct ConversationImport {
    pub source_app: String,
    pub source_conversation_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub sync_strength: String,
    pub raw_metadata_json: String,
}

#[derive(Debug, Clone)]
pub struct MessageImport {
    pub source_message_id: String,
    pub role: String,
    pub message_type: String,
    pub content_text: String,
    pub tool_name: Option<String>,
    pub created_at: i64,
    pub raw_payload_json: String,
}

#[derive(Debug, Clone)]
pub struct ImportedConversation {
    pub conversation: ConversationImport,
    pub messages: Vec<MessageImport>,
}
