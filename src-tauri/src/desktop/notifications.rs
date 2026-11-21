use crate::db::settings;
use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use chrono::{Local, Timelike};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::{NotificationExt, PermissionState};

const SETTINGS_KEY: &str = "desktop.notifications";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub enabled: bool,
    pub feed_refresh: bool,
    pub backup_complete: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_hour_start: u8,
    pub quiet_hour_end: u8,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            feed_refresh: true,
            backup_complete: true,
            quiet_hours_enabled: false,
            quiet_hour_start: 22,
            quiet_hour_end: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStatus {
    pub supported: bool,
    pub permission: String,
    pub settings: NotificationSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEvent {
    FeedRefresh,
    BackupComplete,
}

pub fn normalize(mut value: NotificationSettings) -> AppResult<NotificationSettings> {
    if value.quiet_hour_start > 23 || value.quiet_hour_end > 23 {
        return Err(AppError::Validation(
            "Quiet hours must use whole hours from 0 to 23.".to_string(),
        ));
    }
    if !value.enabled {
        value.quiet_hours_enabled = false;
    }
    Ok(value)
}

pub fn settings(connection: &Connection) -> AppResult<NotificationSettings> {
    let Some(value) = settings::get_global_setting(connection, SETTINGS_KEY)? else {
        return Ok(NotificationSettings::default());
    };
    match serde_json::from_value(value) {
        Ok(value) => normalize(value),
        Err(_) => Ok(NotificationSettings::default()),
    }
}

pub fn update_settings(
    connection: &Connection,
    value: NotificationSettings,
) -> AppResult<NotificationSettings> {
    let value = normalize(value)?;
    settings::set_global_setting(connection, SETTINGS_KEY, &json!(value))?;
    Ok(value)
}

pub fn is_quiet_hour(hour: u32, start: u8, end: u8) -> bool {
    let hour = hour as u8;
    if start == end {
        return false;
    }
    if start < end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}

fn permission_label(state: PermissionState) -> &'static str {
    match state {
        PermissionState::Granted => "granted",
        PermissionState::Denied => "denied",
        PermissionState::Prompt => "prompt",
        PermissionState::PromptWithRationale => "prompt-with-rationale",
    }
}

pub fn status<R: Runtime>(
    app: &AppHandle<R>,
    connection: &Connection,
) -> AppResult<NotificationStatus> {
    let (permission, supported) = match app.notification().permission_state() {
        Ok(state) => (permission_label(state), true),
        Err(_) => ("unavailable", false),
    };
    Ok(NotificationStatus {
        supported,
        permission: permission.to_string(),
        settings: settings(connection)?,
    })
}

pub fn allowed(connection: &Connection, event: NotificationEvent) -> AppResult<bool> {
    let settings = settings(connection)?;
    if !settings.enabled {
        return Ok(false);
    }
    let event_enabled = match event {
        NotificationEvent::FeedRefresh => settings.feed_refresh,
        NotificationEvent::BackupComplete => settings.backup_complete,
    };
    if !event_enabled {
        return Ok(false);
    }
    if settings.quiet_hours_enabled
        && is_quiet_hour(
            Local::now().hour(),
            settings.quiet_hour_start,
            settings.quiet_hour_end,
        )
    {
        return Ok(false);
    }
    Ok(true)
}

pub fn show<R: Runtime>(
    app: &AppHandle<R>,
    connection: &Connection,
    event: NotificationEvent,
    title: &str,
    body: &str,
) -> AppResult<bool> {
    if !allowed(connection, event)? {
        return Ok(false);
    }
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| {
            AppError::Internal(format!(
                "The operating system refused the notification: {}",
                redact(&error.to_string())
            ))
        })?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_hours_handle_daytime_and_midnight_windows() {
        assert!(is_quiet_hour(22, 22, 7));
        assert!(is_quiet_hour(3, 22, 7));
        assert!(!is_quiet_hour(12, 22, 7));
        assert!(is_quiet_hour(9, 9, 17));
        assert!(!is_quiet_hour(18, 9, 17));
        assert!(!is_quiet_hour(12, 12, 12));
    }

    #[test]
    fn disabling_notifications_disables_quiet_hours_without_losing_hours() {
        let value = normalize(NotificationSettings {
            enabled: false,
            quiet_hours_enabled: true,
            quiet_hour_start: 23,
            quiet_hour_end: 6,
            ..NotificationSettings::default()
        })
        .expect("normalize");
        assert!(!value.quiet_hours_enabled);
        assert_eq!(value.quiet_hour_start, 23);
    }

    #[test]
    fn invalid_hours_are_rejected() {
        assert!(normalize(NotificationSettings {
            quiet_hour_start: 24,
            ..NotificationSettings::default()
        })
        .is_err());
    }

    #[test]
    fn settings_are_stored_globally_and_survive_reload() {
        let directory = tempfile::tempdir().expect("temp directory");
        let connection =
            crate::db::initialize(&directory.path().join("bergag.sqlite3")).expect("db");
        let value = NotificationSettings {
            enabled: true,
            feed_refresh: false,
            backup_complete: true,
            quiet_hours_enabled: true,
            quiet_hour_start: 1,
            quiet_hour_end: 5,
        };
        let saved = update_settings(&connection, value.clone()).expect("save");
        assert_eq!(saved, value);
        assert_eq!(settings(&connection).expect("reload"), value);
    }
}
