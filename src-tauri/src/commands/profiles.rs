use super::{command_error, locked_connection};
use crate::db::{models::ProfileDto, profiles};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn profiles_list(state: State<'_, AppState>) -> Result<Vec<ProfileDto>, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    profiles::list_profiles(&connection).map_err(command_error)
}

#[tauri::command]
pub fn profiles_get_active(state: State<'_, AppState>) -> Result<ProfileDto, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    profiles::get_active_profile(&connection).map_err(command_error)
}

#[tauri::command]
pub fn profiles_create(name: String, state: State<'_, AppState>) -> Result<ProfileDto, String> {
    let mut connection = locked_connection(&state).map_err(command_error)?;
    profiles::create_profile(&mut connection, &name).map_err(command_error)
}

#[tauri::command]
pub fn profiles_rename(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<ProfileDto, String> {
    let mut connection = locked_connection(&state).map_err(command_error)?;
    profiles::rename_profile(&mut connection, &id, &name).map_err(command_error)
}

#[tauri::command]
pub fn profiles_switch(id: String, state: State<'_, AppState>) -> Result<ProfileDto, String> {
    let mut connection = locked_connection(&state).map_err(command_error)?;
    profiles::switch_profile(&mut connection, &id).map_err(command_error)
}

#[tauri::command]
pub fn profiles_delete(id: String, state: State<'_, AppState>) -> Result<Vec<ProfileDto>, String> {
    let mut connection = locked_connection(&state).map_err(command_error)?;
    crate::backup::service::create(
        &state.backup_dir,
        &connection,
        Some("Before profile deletion"),
        true,
    )
    .map_err(command_error)?;
    profiles::delete_profile(&mut connection, &id).map_err(command_error)
}
