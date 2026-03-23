use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

pub fn export_note_markdown(connection: &Connection, note_id: &str, path: &Path) -> Result<()> {
    let body_md: String = connection.query_row(
        "SELECT body_md FROM notes WHERE id = ?1",
        [note_id],
        |row| row.get(0),
    )?;
    std::fs::write(path, body_md)?;
    Ok(())
}
