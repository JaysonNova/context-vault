use std::path::Path;

use rusqlite::Connection;

use crate::db::migrations::run_migrations;
use crate::error::AppResult;

pub fn open_and_migrate<P: AsRef<Path>>(path: P) -> AppResult<Connection> {
    let connection = Connection::open(path)?;
    run_migrations(&connection)?;
    Ok(connection)
}
