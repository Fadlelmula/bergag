use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub available: bool,
    pub enabled: bool,
    pub detail: String,
}

fn unavailable() -> StartupStatus {
    StartupStatus {
        available: false,
        enabled: false,
        detail: "Launch-on-sign-in is unavailable on this platform.".to_string(),
    }
}

pub fn status(app: &AppHandle) -> AppResult<StartupStatus> {
    if !cfg!(windows) {
        return Ok(unavailable());
    }
    let enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| AppError::Internal(format!("Could not read startup state: {error}")))?;
    Ok(StartupStatus {
        available: true,
        enabled,
        detail: if enabled {
            "Bergag is registered to launch when you sign in.".to_string()
        } else {
            "Bergag does not launch automatically.".to_string()
        },
    })
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> AppResult<StartupStatus> {
    if !cfg!(windows) {
        return Ok(unavailable());
    }
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|error| {
            AppError::Internal(format!("Could not enable startup launch: {error}"))
        })?;
    } else {
        manager.disable().map_err(|error| {
            AppError::Internal(format!("Could not disable startup launch: {error}"))
        })?;
    }
    status(app)
}
