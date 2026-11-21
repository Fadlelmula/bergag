use crate::logging::LoggingGuard;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub connection: Mutex<Connection>,
    pub db_path: PathBuf,
    pub backup_dir: PathBuf,
    pub log_dir: PathBuf,
    #[allow(dead_code)]
    pub log_guard: Mutex<Option<LoggingGuard>>,
}
