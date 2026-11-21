use super::{notifications, settings};
use crate::error::{AppError, AppResult};
use crate::feeds::service as feed_service;
use crate::security::redaction::redact;
use crate::state::AppState;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Emitter, Manager, WindowEvent};

const SHOW_ID: &str = "tray.show";
const REFRESH_ID: &str = "tray.refresh";
const BACKUP_ID: &str = "tray.backup";
const QUIT_ID: &str = "tray.quit";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrayStatus {
    pub available: bool,
    pub close_behavior: settings::CloseBehavior,
}

pub fn status(connection: &Connection) -> AppResult<TrayStatus> {
    Ok(TrayStatus {
        available: cfg!(windows),
        close_behavior: settings::close_behavior(connection)?,
    })
}

fn tauri_error(context: &str, error: tauri::Error) -> AppError {
    AppError::Internal(format!("{context}: {}", redact(&error.to_string())))
}

pub fn setup(app: &App) -> AppResult<()> {
    if !cfg!(windows) {
        return Ok(());
    }

    let show = MenuItem::with_id(app, SHOW_ID, "Show Bergag", true, None::<&str>)
        .map_err(|e| tauri_error("tray menu", e))?;
    let refresh = MenuItem::with_id(app, REFRESH_ID, "Refresh due feeds", true, None::<&str>)
        .map_err(|e| tauri_error("tray menu", e))?;
    let backup = MenuItem::with_id(app, BACKUP_ID, "Create local backup", true, None::<&str>)
        .map_err(|e| tauri_error("tray menu", e))?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit Bergag", true, None::<&str>)
        .map_err(|e| tauri_error("tray menu", e))?;
    let separator = PredefinedMenuItem::separator(app).map_err(|e| tauri_error("tray menu", e))?;
    let menu = Menu::with_items(app, &[&show, &refresh, &backup, &separator, &quit])
        .map_err(|e| tauri_error("tray menu", e))?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| AppError::Internal("The application icon is unavailable.".to_string()))?;

    let tray = TrayIconBuilder::with_id("bergag-main-tray")
        .icon(icon)
        .tooltip("Bergag — local reading platform")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_ID => show_window(app),
            REFRESH_ID => refresh_feeds(app),
            BACKUP_ID => create_backup(app),
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)
        .map_err(|e| tauri_error("tray icon", e))?;
    app.manage(tray);

    if let Some(window) = app.get_webview_window("main") {
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let close_behavior = app_handle
                    .state::<AppState>()
                    .connection
                    .lock()
                    .map_err(|_| ())
                    .ok()
                    .and_then(|connection| settings::close_behavior(&connection).ok());
                if close_behavior == Some(settings::CloseBehavior::Tray) {
                    api.prevent_close();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            }
        });
    }

    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn refresh_feeds(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let outcomes = {
            let state = app_handle.state::<AppState>();
            feed_service::refresh_due(state.inner()).await
        };
        match outcomes {
            Ok(outcomes) => {
                let count = outcomes.len();
                if count > 0 {
                    if let Err(error) = app_handle.emit("feeds:refreshed", outcomes) {
                        tracing::warn!(error = %redact(&error.to_string()), "tray refresh event failed");
                    }
                }
                notify_result(
                    &app_handle,
                    notifications::NotificationEvent::FeedRefresh,
                    if count == 0 {
                        "No feeds were due"
                    } else {
                        "Feed refresh finished"
                    },
                    if count == 0 {
                        "Bergag checked the subscription schedule; nothing was due."
                    } else {
                        "Bergag refreshed the feeds that were due."
                    },
                );
            }
            Err(error) => tracing::warn!(
                error = %redact(&error.to_string()),
                "tray feed refresh failed"
            ),
        }
    });
}

fn create_backup(app: &AppHandle) {
    let state = app.state::<AppState>();
    let result = {
        let connection = match state.connection.lock() {
            Ok(connection) => connection,
            Err(_) => return,
        };
        crate::backup::service::create(&state.backup_dir, &connection, Some("Tray backup"), true)
    };
    match result {
        Ok(backup) => notify_result(
            app,
            notifications::NotificationEvent::BackupComplete,
            "Backup created",
            &format!("Bergag stored \"{}\" locally.", backup.label),
        ),
        Err(error) => tracing::warn!(error = %redact(&error.to_string()), "tray backup failed"),
    }
}

fn notify_result(
    app: &AppHandle,
    event: notifications::NotificationEvent,
    title: &str,
    body: &str,
) {
    let state = app.state::<AppState>();
    let connection = match state.connection.lock() {
        Ok(connection) => connection,
        Err(_) => return,
    };
    if let Err(error) = notifications::show(app, &connection, event, title, body) {
        tracing::warn!(error = %redact(&error.to_string()), "tray notification failed");
    }
}
