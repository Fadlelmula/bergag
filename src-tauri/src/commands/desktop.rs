use super::{command_error, locked_connection};
use crate::desktop::{notifications, settings, startup, tray, updates};
use crate::error::AppError;
use crate::security::status as security_status;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn desktop_notifications_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<notifications::NotificationStatus, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    notifications::status(&app, &connection).map_err(command_error)
}

#[tauri::command]
pub fn desktop_notifications_update(
    settings_value: notifications::NotificationSettings,
    state: State<'_, AppState>,
) -> Result<notifications::NotificationSettings, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    notifications::update_settings(&connection, settings_value).map_err(command_error)
}

#[tauri::command]
pub fn desktop_notifications_test(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    let shown = notifications::show(
        &app,
        &connection,
        notifications::NotificationEvent::BackupComplete,
        "Bergag notification test",
        "Bergag can send local desktop notifications.",
    )
    .map_err(command_error)?;
    if shown {
        Ok(true)
    } else {
        Err(command_error(AppError::Validation(
            "Notifications are disabled, denied, or suppressed by quiet hours. No notification was sent."
                .to_string(),
        )))
    }
}

#[tauri::command]
pub fn desktop_tray_status(state: State<'_, AppState>) -> Result<tray::TrayStatus, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    tray::status(&connection).map_err(command_error)
}

#[tauri::command]
pub fn desktop_tray_set_close_behavior(
    behavior: settings::CloseBehavior,
    state: State<'_, AppState>,
) -> Result<tray::TrayStatus, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    settings::set_close_behavior(&connection, behavior).map_err(command_error)?;
    tray::status(&connection).map_err(command_error)
}

#[tauri::command]
pub fn desktop_startup_status(app: AppHandle) -> Result<startup::StartupStatus, String> {
    startup::status(&app).map_err(command_error)
}

#[tauri::command]
pub fn desktop_startup_set_enabled(
    enabled: bool,
    app: AppHandle,
) -> Result<startup::StartupStatus, String> {
    startup::set_enabled(&app, enabled).map_err(command_error)
}

#[tauri::command]
pub fn desktop_updates_status() -> Result<updates::UpdateStatus, String> {
    Ok(updates::status())
}

#[tauri::command]
pub async fn desktop_updates_check(app: AppHandle) -> Result<updates::UpdateCheck, String> {
    updates::check(&app).await.map_err(command_error)
}

#[tauri::command]
pub fn desktop_security_status(
    state: State<'_, AppState>,
) -> Result<security_status::SecurityStatus, String> {
    let connection = locked_connection(&state).map_err(command_error)?;
    security_status::status(&connection).map_err(command_error)
}
