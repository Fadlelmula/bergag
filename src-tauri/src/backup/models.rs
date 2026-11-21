use serde::{Deserialize, Serialize};

pub const BACKUP_FORMAT: &str = "bergag-backup-v1";
pub const MANIFEST_SUFFIX: &str = ".manifest.json";
pub const DATABASE_SUFFIX: &str = ".sqlite3";
pub const AUTOMATIC_RETENTION: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupManifest {
    pub format: String,
    pub app_version: String,
    pub schema_version: u32,
    pub created_at: String,
    pub profile_count: usize,
    pub profile_ids: Vec<String>,
    pub database_sha256: String,
    pub database_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupRecord {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub schema_version: u32,
    pub profile_count: usize,
    pub size_bytes: u64,
    pub automatic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupVerification {
    pub valid: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub safety_backup_id: String,
    pub schema_version: u32,
    pub profile_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BackupIndex {
    pub(crate) records: Vec<BackupRecord>,
}
