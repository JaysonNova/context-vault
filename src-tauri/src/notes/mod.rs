pub mod prompt;
pub mod provider;

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::notes::prompt::{build_prompt, PROMPT_VERSION};
use crate::notes::provider::NoteProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub body_md: String,
    pub prompt_version: String,
}

pub fn generate_note<P: NoteProvider>(
    connection: &Connection,
    provider: P,
    conversation_id: &str,
) -> Result<NoteRecord> {
    let title: String = connection.query_row(
        "SELECT title FROM conversations WHERE id = ?1",
        [conversation_id],
        |row| row.get(0),
    )?;

    let mut statement = connection.prepare(
        "SELECT content_text FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let messages = statement
        .query_map([conversation_id], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    if messages.is_empty() {
        return Err(anyhow!("cannot generate note without imported messages"));
    }

    let prompt = build_prompt(&title, &messages);
    let body_md = provider.generate(&prompt)?;
    let summary = body_md
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .unwrap_or("Generated technical note")
        .to_string();
    let note = NoteRecord {
        id: Uuid::new_v4().to_string(),
        title,
        summary,
        body_md,
        prompt_version: PROMPT_VERSION.to_string(),
    };

    let now = OffsetDateTime::now_utc().unix_timestamp_nanos() as i64 / 1_000_000;
    connection.execute(
        "INSERT INTO notes (
            id, title, summary, body_md, status, created_at, updated_at,
            model_provider, model_name, prompt_version
        ) VALUES (?1, ?2, ?3, ?4, 'ready', ?5, ?5, 'stub', 'stub', ?6)",
        params![
            note.id,
            note.title,
            note.summary,
            note.body_md,
            now,
            note.prompt_version
        ],
    )?;
    connection.execute(
        "INSERT INTO note_sources (note_id, conversation_id) VALUES (?1, ?2)",
        params![note.id, conversation_id],
    )?;

    Ok(note)
}
