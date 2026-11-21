pub mod diagnostics;
pub mod feeds;
pub mod layouts;
pub mod migrations;
pub mod models;
pub mod profiles;
pub mod settings;

use crate::error::AppResult;
use rusqlite::Connection;
use std::path::Path;

pub fn initialize(path: &Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            crate::error::AppError::Internal(format!("Could not create data directory: {error}"))
        })?;
    }

    let mut connection = Connection::open(path)?;
    configure(&connection)?;
    let applied = migrations::run(&mut connection)?;
    profiles::ensure_default_profile(&mut connection)?;
    tracing::info!(
        path = %path.display(),
        migrations_applied = applied,
        "Bergag database initialized"
    );
    Ok(connection)
}

pub fn configure(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )?;
    Ok(())
}
