use super::{command_error, locked_connection};
use crate::privacy::{self, PrivacyDashboardDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn privacy_dashboard_get(state: State<'_, AppState>) -> Result<PrivacyDashboardDto, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    privacy::dashboard(
        &connection,
        &state.db_path,
        &state.backup_dir,
        &state.log_dir,
    )
    .map_err(command_error)
}
