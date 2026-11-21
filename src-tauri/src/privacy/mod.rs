use crate::backup;
use crate::db::{diagnostics, settings};
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyDashboardDto {
    pub app_version: String,
    pub schema_version: u32,
    pub telemetry_enabled: bool,
    pub ai_enabled: bool,
    pub ai_provider_count: usize,
    pub remote_ai_provider_count: usize,
    pub local_ai_provider_count: usize,
    pub ai_usage_event_count: usize,
    pub integration_source_count: usize,
    pub feed_count: usize,
    pub article_count: usize,
    pub note_count: usize,
    pub highlight_count: usize,
    pub label_count: usize,
    pub rule_count: usize,
    pub dashboard_count: usize,
    pub profile_count: usize,
    pub backup_count: usize,
    pub database_path: String,
    pub backup_directory: String,
    pub log_directory: String,
    pub integrity_check: String,
}

fn count(
    connection: &Connection,
    sql: &str,
    parameters: &[&dyn rusqlite::ToSql],
) -> AppResult<usize> {
    let value: i64 = connection.query_row(sql, parameters, |row| row.get(0))?;
    usize::try_from(value)
        .map_err(|_| AppError::Internal("A local privacy count is out of range.".to_string()))
}

pub fn dashboard(
    connection: &Connection,
    db_path: &Path,
    backup_dir: &Path,
    log_dir: &Path,
) -> AppResult<PrivacyDashboardDto> {
    let report = diagnostics::diagnostics(connection, db_path, log_dir)?;
    let profile_id = settings::get_active_profile_id(connection)?;
    let ai_enabled = settings::get_profile_setting(connection, &profile_id, "ai.settings")?
        .and_then(|value| value.get("enabled")?.as_bool())
        .unwrap_or(false);

    let remote_ai_provider_count = count(
        connection,
        "SELECT COUNT(*) FROM ai_providers WHERE profile_id = ?1 AND kind != 'ollama'",
        params![profile_id.as_str()],
    )?;
    let local_ai_provider_count = count(
        connection,
        "SELECT COUNT(*) FROM ai_providers WHERE profile_id = ?1 AND kind = 'ollama'",
        params![profile_id.as_str()],
    )?;

    Ok(PrivacyDashboardDto {
        app_version: report.app_version,
        schema_version: u32::try_from(report.database.user_version).unwrap_or(0),
        telemetry_enabled: report.telemetry_enabled,
        ai_enabled,
        ai_provider_count: remote_ai_provider_count + local_ai_provider_count,
        remote_ai_provider_count,
        local_ai_provider_count,
        ai_usage_event_count: count(
            connection,
            "SELECT COUNT(*) FROM ai_usage_events WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        integration_source_count: count(
            connection,
            "SELECT COUNT(*) FROM integration_sources WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        feed_count: count(
            connection,
            "SELECT COUNT(*) FROM feeds WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        article_count: count(
            connection,
            "SELECT COUNT(*) FROM feed_items WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        note_count: count(
            connection,
            "SELECT COUNT(*) FROM article_notes WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        highlight_count: count(
            connection,
            "SELECT COUNT(*) FROM article_highlights WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        label_count: count(
            connection,
            "SELECT COUNT(*) FROM labels WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        rule_count: count(
            connection,
            "SELECT COUNT(*) FROM rules WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        dashboard_count: count(
            connection,
            "SELECT COUNT(*) FROM dashboards WHERE profile_id = ?1",
            params![profile_id.as_str()],
        )?,
        profile_count: count(connection, "SELECT COUNT(*) FROM profiles", &[])?,
        backup_count: backup::service::list(backup_dir)?.len(),
        database_path: report.database.path,
        backup_directory: backup_dir.to_string_lossy().to_string(),
        log_directory: report.log_dir,
        integrity_check: report.database.integrity_check,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::fs;

    #[test]
    fn privacy_dashboard_reports_real_local_state_without_seeded_content() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let backup_dir = directory.path().join("backups");
        let log_dir = directory.path().join("logs");
        fs::create_dir_all(&log_dir).expect("log dir");
        let connection = db::initialize(&db_path).expect("database");
        backup::service::initialize(&backup_dir).expect("backups");

        let report = dashboard(&connection, &db_path, &backup_dir, &log_dir).expect("report");
        assert!(!report.telemetry_enabled);
        assert!(!report.ai_enabled);
        assert_eq!(
            report.schema_version,
            crate::db::migrations::current_version()
        );
        assert_eq!(report.profile_count, 1);
        assert_eq!(report.backup_count, 0);
        assert_eq!(report.feed_count, 0);
        assert_eq!(report.article_count, 0);
        assert_eq!(report.ai_provider_count, 0);
        assert_eq!(report.integration_source_count, 0);
        assert!(!report.database_path.is_empty());
        assert!(!report.backup_directory.is_empty());
        assert_eq!(report.integrity_check, "ok");
    }
}
