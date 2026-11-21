use super::models::{DbInfoDto, DiagnosticsDto};
use crate::error::AppResult;
use rusqlite::Connection;
use std::path::Path;

pub fn probe_fts5(connection: &Connection) -> bool {
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.bergag_fts_probe USING fts5(content);
             DROP TABLE temp.bergag_fts_probe;",
        )
        .is_ok()
}

pub fn db_info(connection: &Connection, path: &Path) -> AppResult<DbInfoDto> {
    let integrity_check: String =
        connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    let sqlite_version: String =
        connection.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
    let migration_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
    let user_version: i64 =
        connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let size_bytes = std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    Ok(DbInfoDto {
        path: path.to_string_lossy().to_string(),
        size_bytes,
        sqlite_version,
        migration_count,
        user_version,
        integrity_check,
        fts5_available: probe_fts5(connection),
    })
}

pub fn diagnostics(
    connection: &Connection,
    db_path: &Path,
    log_dir: &Path,
) -> AppResult<DiagnosticsDto> {
    Ok(DiagnosticsDto {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: tauri::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        telemetry_enabled: false,
        log_dir: log_dir.to_string_lossy().to_string(),
        database: db_info(connection, db_path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn reports_real_database_information() {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        db::profiles::ensure_default_profile(&mut connection).unwrap();

        // FTS5 may be unavailable in some bundled SQLite builds; the probe must not fail.
        let _ = probe_fts5(&connection);

        let info = db_info(&connection, Path::new(":memory:")).unwrap();
        assert!(info.sqlite_version.contains('.'));
        assert_eq!(info.integrity_check, "ok");
        assert!(info.migration_count >= 1);

        let report = diagnostics(&connection, Path::new(":memory:"), Path::new("logs")).unwrap();
        assert_eq!(report.tauri_version, tauri::VERSION);
        assert!(!report.telemetry_enabled);
    }
}
