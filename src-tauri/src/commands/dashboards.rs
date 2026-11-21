use super::{active_profile_id, command_error, locked_connection};
use crate::dashboards::{models, store};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn dashboards_list(state: State<'_, AppState>) -> Result<Vec<models::DashboardDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_dashboards(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn dashboard_save(
    id: Option<String>,
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::DashboardDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::save_dashboard(
        &mut connection,
        &profile_id,
        id.as_deref(),
        &name,
        description.as_deref(),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn dashboard_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::delete_dashboard(&mut connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn dashboard_set_default(
    id: String,
    state: State<'_, AppState>,
) -> Result<models::DashboardDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::set_default_dashboard(&mut connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn widgets_list(
    dashboard_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::DashboardWidgetDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_widgets(&connection, &profile_id, &dashboard_id).map_err(command_error)
}

#[tauri::command]
pub fn widget_add(
    dashboard_id: String,
    kind: String,
    state: State<'_, AppState>,
) -> Result<models::DashboardWidgetDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::add_widget(&mut connection, &profile_id, &dashboard_id, &kind).map_err(command_error)
}

#[tauri::command]
pub fn widget_update(
    id: String,
    title: String,
    days: i64,
    item_limit: i64,
    sort_order: i64,
    state: State<'_, AppState>,
) -> Result<models::DashboardWidgetDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::update_widget(
        &connection,
        &profile_id,
        &id,
        &title,
        days,
        item_limit,
        sort_order,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn widget_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_widget(&connection, &profile_id, &id).map_err(command_error)
}
