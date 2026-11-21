use super::{active_profile_id, command_error, locked_connection};
use crate::db::{layouts, models::LayoutRecordDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn layouts_list(state: State<'_, AppState>) -> Result<Vec<LayoutRecordDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    layouts::list_layouts(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn layouts_get_active(state: State<'_, AppState>) -> Result<Option<LayoutRecordDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    layouts::get_active_layout(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn layouts_save(
    name: String,
    layout_json: String,
    activate: Option<bool>,
    state: State<'_, AppState>,
) -> Result<LayoutRecordDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    layouts::save_layout(
        &mut connection,
        &profile_id,
        &name,
        &layout_json,
        activate.unwrap_or(true),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn layouts_activate(id: String, state: State<'_, AppState>) -> Result<LayoutRecordDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    layouts::activate_layout(&mut connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn layouts_delete(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<LayoutRecordDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    layouts::delete_layout(&mut connection, &profile_id, &id).map_err(command_error)
}
