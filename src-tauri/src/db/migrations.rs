use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATION_0001_INIT: &str = include_str!("../../migrations/0001_init.sql");
const MIGRATION_0002_FTS: &str = include_str!("../../migrations/0002_fts.sql");
const MIGRATION_0003_PERFORMANCE_INDEXES: &str =
    include_str!("../../migrations/0003_performance_indexes.sql");

pub fn run_migrations(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(MIGRATION_0001_INIT)?;
    connection.execute_batch(MIGRATION_0002_FTS)?;
    connection.execute_batch(MIGRATION_0003_PERFORMANCE_INDEXES)?;
    Ok(())
}
