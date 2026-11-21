use super::{active_profile_id, command_error, locked_connection};
use crate::db::settings;
use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn settings_get_profile(
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<Value>, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    let profile_id = settings::get_active_profile_id(&connection).map_err(command_error)?;
    settings::get_profile_setting(&connection, &profile_id, &key).map_err(command_error)
}

#[tauri::command]
pub fn settings_set_profile(
    key: String,
    value: Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    let profile_id = settings::get_active_profile_id(&connection).map_err(command_error)?;
    settings::set_profile_setting(&connection, &profile_id, &key, &value).map_err(command_error)
}

#[tauri::command]
pub fn settings_delete_profile(key: String, state: State<'_, AppState>) -> Result<(), String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    let profile_id = settings::get_active_profile_id(&connection).map_err(command_error)?;
    settings::delete_profile_setting(&connection, &profile_id, &key).map_err(command_error)
}

#[tauri::command]
pub fn settings_get_active_profile_id(state: State<'_, AppState>) -> Result<String, String> {
    active_profile_id(&state).map_err(command_error)
}
