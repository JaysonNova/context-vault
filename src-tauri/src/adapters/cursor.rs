use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::sync::normalize::{ConversationImport, ImportedConversation, MessageImport};

pub struct CursorAdapter;

#[derive(Debug, Deserialize, Serialize)]
struct ComposerData {
    #[serde(rename = "allComposers")]
    all_composers: Vec<ComposerHead>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ComposerHead {
    #[serde(rename = "composerId")]
    composer_id: String,
    name: Option<String>,
    subtitle: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "lastUpdatedAt")]
    last_updated_at: i64,
    #[serde(rename = "createdOnBranch")]
    created_on_branch: Option<String>,
    #[serde(rename = "filesChangedCount")]
    files_changed_count: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GenerationEntry {
    #[serde(rename = "unixMs")]
    unix_ms: i64,
    #[serde(rename = "generationUUID")]
    generation_uuid: String,
    #[serde(rename = "composerId")]
    composer_id: Option<String>,
    #[serde(rename = "textDescription")]
    text_description: Option<String>,
}

impl CursorAdapter {
    pub fn parse_fixture_dir<P: AsRef<Path>>(fixture_dir: P) -> Result<ImportedConversation> {
        let fixture_dir = fixture_dir.as_ref();
        let composer_data: ComposerData =
            serde_json::from_str(&fs::read_to_string(fixture_dir.join("composer_data.json"))?)?;
        let generations: Vec<GenerationEntry> =
            serde_json::from_str(&fs::read_to_string(fixture_dir.join("generations.json"))?)?;

        let composer = composer_data
            .all_composers
            .first()
            .ok_or_else(|| anyhow!("missing composer fixture"))?;

        let generation_hint = generations
            .iter()
            .find(|entry| entry.composer_id.as_deref() == Some(composer.composer_id.as_str()))
            .and_then(|entry| entry.text_description.clone());

        let conversation = ConversationImport {
            source_app: "cursor".into(),
            source_conversation_id: composer.composer_id.clone(),
            title: composer
                .name
                .clone()
                .unwrap_or_else(|| "Untitled Cursor Composer".into()),
            subtitle: composer.subtitle.clone(),
            created_at: composer.created_at,
            updated_at: composer.last_updated_at,
            sync_strength: "metadata_only".into(),
            raw_metadata_json: json!({
                "composer": composer,
                "generation_hint": generation_hint
            })
            .to_string(),
        };

        let messages: Vec<MessageImport> = generation_hint
            .map(|text| MessageImport {
                source_message_id: format!("{}:generation", composer.composer_id),
                role: "system".into(),
                message_type: "metadata".into(),
                content_text: text,
                tool_name: None,
                created_at: composer.last_updated_at,
                raw_payload_json: json!({
                    "composer_id": composer.composer_id,
                    "kind": "generation_hint"
                })
                .to_string(),
            })
            .into_iter()
            .collect();

        Ok(ImportedConversation {
            conversation,
            messages,
        })
    }
}
