use super::{command_error, locked_connection};
use crate::backup::models::{BackupRecord, BackupVerification, RestoreOutcome};
use crate::backup::service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn backup_create(
    label: Option<String>,
    state: State<'_, AppState>,
) -> Result<BackupRecord, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    service::create(&state.backup_dir, &connection, label.as_deref(), false).map_err(command_error)
}

#[tauri::command]
pub fn backup_list(state: State<'_, AppState>) -> Result<Vec<BackupRecord>, String> {
    service::list(&state.backup_dir).map_err(command_error)
}

#[tauri::command]
pub fn backup_verify(id: String, state: State<'_, AppState>) -> Result<BackupVerification, String> {
    service::verify(&state.backup_dir, &id).map_err(command_error)
}

#[tauri::command]
pub fn backup_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    service::delete(&state.backup_dir, &id).map_err(command_error)
}

#[tauri::command]
pub fn backup_restore(id: String, state: State<'_, AppState>) -> Result<RestoreOutcome, String> {
    let mut connection = locked_connection(&state).map_err(command_error)?;
    service::restore(&state.backup_dir, &mut connection, &id).map_err(command_error)
}
