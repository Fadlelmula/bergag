use super::{command_error, locked_connection};
use crate::db::{diagnostics, models::DiagnosticsDto};
use crate::logging;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn diagnostics_get(state: State<'_, AppState>) -> Result<DiagnosticsDto, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    diagnostics::diagnostics(&connection, &state.db_path, &state.log_dir).map_err(command_error)
}

#[tauri::command]
pub fn diagnostics_recent_logs(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let limit = limit.unwrap_or(50).clamp(1, 200);
    logging::recent_logs(&state.log_dir, limit).map_err(command_error)
}
