use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATION_0001_INIT: &str = include_str!("../../migrations/0001_init.sql");
const MIGRATION_0002_FTS: &str = include_str!("../../migrations/0002_fts.sql");

pub fn run_migrations(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(MIGRATION_0001_INIT)?;
    connection.execute_batch(MIGRATION_0002_FTS)?;
    Ok(())
}
