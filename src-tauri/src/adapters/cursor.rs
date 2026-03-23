use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use glob::glob;
use rusqlite::Connection;
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
    created_at: Option<i64>,
    #[serde(rename = "lastUpdatedAt")]
    last_updated_at: Option<i64>,
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
            created_at: composer.created_at.unwrap_or_else(|| composer.last_updated_at.unwrap_or(0)),
            updated_at: composer.last_updated_at.unwrap_or_else(|| composer.created_at.unwrap_or(0)),
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
                created_at: composer
                    .last_updated_at
                    .unwrap_or_else(|| composer.created_at.unwrap_or(0)),
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

    pub fn import_default_sources() -> Result<Vec<ImportedConversation>> {
        let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
        let pattern = format!(
            "{home}/Library/Application Support/Cursor/User/workspaceStorage/**/state.vscdb"
        );
        let mut conversations = Vec::new();

        for path in glob(&pattern)? {
            let path = path?;
            match Self::parse_workspace_db(path) {
                Ok(items) => conversations.extend(items),
                Err(error) if error.to_string().contains("Query returned no rows") => continue,
                Err(error) => return Err(error),
            }
        }

        Ok(conversations)
    }

    fn parse_workspace_db<P: AsRef<Path>>(path: P) -> Result<Vec<ImportedConversation>> {
        let connection = Connection::open(path)?;
        let composer_json: String = connection.query_row(
            "SELECT value FROM ItemTable WHERE key = 'composer.composerData'",
            [],
            |row| row.get(0),
        )?;
        let composer_data: ComposerData = serde_json::from_str(&composer_json)?;
        let generations_json: Option<String> = connection
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'aiService.generations'",
                [],
                |row| row.get(0),
            )
            .ok();
        let generations: Vec<GenerationEntry> = generations_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?
            .unwrap_or_default();

        let mut conversations = Vec::new();
        for composer in composer_data.all_composers {
            let generation_hint = generations
                .iter()
                .find(|entry| entry.composer_id.as_deref() == Some(composer.composer_id.as_str()))
                .and_then(|entry| entry.text_description.clone());

            let messages: Vec<MessageImport> = generation_hint
                .clone()
                .map(|text| MessageImport {
                    source_message_id: format!("{}:generation", composer.composer_id),
                    role: "system".into(),
                    message_type: "metadata".into(),
                    content_text: text,
                    tool_name: None,
                    created_at: composer
                        .last_updated_at
                        .unwrap_or_else(|| composer.created_at.unwrap_or(0)),
                    raw_payload_json: json!({
                        "composer_id": composer.composer_id,
                        "kind": "generation_hint"
                    })
                    .to_string(),
                })
                .into_iter()
                .collect();

            conversations.push(ImportedConversation {
                conversation: ConversationImport {
                    source_app: "cursor".into(),
                    source_conversation_id: composer.composer_id.clone(),
                    title: composer
                        .name
                        .clone()
                        .unwrap_or_else(|| "Untitled Cursor Composer".into()),
                    subtitle: composer.subtitle.clone(),
                    created_at: composer
                        .created_at
                        .unwrap_or_else(|| composer.last_updated_at.unwrap_or(0)),
                    updated_at: composer
                        .last_updated_at
                        .unwrap_or_else(|| composer.created_at.unwrap_or(0)),
                    sync_strength: "metadata_only".into(),
                    raw_metadata_json: json!({
                        "composer": composer,
                        "generation_hint": generation_hint
                    })
                    .to_string(),
                },
                messages,
            });
        }

        Ok(conversations)
    }
}
