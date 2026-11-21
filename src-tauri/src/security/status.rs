use crate::error::{AppError, AppResult};
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityStatus {
    pub profile_lock_available: bool,
    pub locked_profile_count: usize,
    pub lock_detail: String,
    pub credential_store: String,
    pub credential_getter_command: bool,
    pub sqlite_secret_columns: usize,
    pub public_url_validation: bool,
    pub html_sanitization: bool,
    pub log_redaction: bool,
    pub generic_shell_or_filesystem_access: bool,
}

fn sensitive_column(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "api_key" | "apikey" | "password" | "secret" | "token" | "credential" | "authorization"
    )
}

fn sqlite_secret_columns(connection: &Connection) -> AppResult<usize> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let tables = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut count = 0;
    for table in tables {
        let sql = format!("PRAGMA table_info({table})");
        let mut columns = connection.prepare(&sql)?;
        let names = columns
            .query_map([], |row: &Row<'_>| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        count += names.iter().filter(|name| sensitive_column(name)).count();
    }
    Ok(count)
}

pub fn status(connection: &Connection) -> AppResult<SecurityStatus> {
    let locked_profile_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM profiles WHERE is_locked = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(SecurityStatus {
        profile_lock_available: false,
        locked_profile_count: usize::try_from(locked_profile_count).map_err(|_| {
            AppError::Internal("A local security count is out of range.".to_string())
        })?,
        lock_detail: "The is_locked and lock_hint fields are dormant schema fields. Bergag does not encrypt or restrict profile access, so no lock control is shown.".to_string(),
        credential_store: if cfg!(windows) {
            "Windows Credential Manager (write and clear only; no read command)".to_string()
        } else {
            "Platform credential store unavailable in this Windows-first build".to_string()
        },
        credential_getter_command: false,
        sqlite_secret_columns: sqlite_secret_columns(connection)?,
        public_url_validation: true,
        html_sanitization: true,
        log_redaction: true,
        generic_shell_or_filesystem_access: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn security_status_reports_dormant_lock_and_no_sqlite_secrets() {
        let directory = tempfile::tempdir().expect("temp directory");
        let connection =
            db::initialize(&directory.path().join("bergag.sqlite3")).expect("database");
        let report = status(&connection).expect("status");
        assert!(!report.profile_lock_available);
        assert_eq!(report.locked_profile_count, 0);
        assert!(!report.credential_getter_command);
        assert_eq!(report.sqlite_secret_columns, 0);
        assert!(!report.generic_shell_or_filesystem_access);
        assert!(report.lock_detail.contains("does not encrypt"));
    }

    #[test]
    fn sensitive_column_matching_is_exact() {
        assert!(sensitive_column("API_KEY"));
        assert!(sensitive_column("password"));
        assert!(!sensitive_column("prompt_tokens"));
        assert!(!sensitive_column("lock_hint"));
    }
}
