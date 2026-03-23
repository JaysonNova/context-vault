use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::migrations::run_migrations;
use crate::error::AppResult;

pub fn open_and_migrate<P: AsRef<Path>>(path: P) -> AppResult<Connection> {
    let connection = Connection::open(path)?;
    run_migrations(&connection)?;
    Ok(connection)
}

pub fn default_storage_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".context-vault")
}

pub fn default_db_path() -> PathBuf {
    default_storage_dir().join("context-vault.sqlite")
}

pub fn open_default_db() -> AppResult<Connection> {
    let storage_dir = default_storage_dir();
    fs::create_dir_all(&storage_dir)?;
    open_and_migrate(default_db_path())
}
