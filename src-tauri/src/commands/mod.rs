pub mod ai;
pub mod analytics;
pub mod articles;
pub mod backups;
pub mod dashboards;
pub mod desktop;
pub mod diagnostics;
pub mod discovery;
pub mod feeds;
pub mod integrations;
pub mod layouts;
pub mod privacy;
pub mod profiles;
pub mod search;
pub mod settings;

pub use crate::error::command_error;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

pub(crate) fn locked_connection<'a>(
    state: &'a State<'a, AppState>,
) -> AppResult<MutexGuard<'a, Connection>> {
    state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })
}

pub(crate) fn active_profile_id(state: &State<'_, AppState>) -> AppResult<String> {
    let connection = locked_connection(state)?;
    crate::db::settings::get_active_profile_id(&connection)
}
