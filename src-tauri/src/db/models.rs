use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileDto {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_locked: bool,
    pub lock_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutRecordDto {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub layout_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbInfoDto {
    pub path: String,
    pub size_bytes: u64,
    pub sqlite_version: String,
    pub migration_count: i64,
    pub user_version: i64,
    pub integrity_check: String,
    pub fts5_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticsDto {
    pub app_version: String,
    pub tauri_version: String,
    pub os: String,
    pub arch: String,
    pub telemetry_enabled: bool,
    pub log_dir: String,
    pub database: DbInfoDto,
}
