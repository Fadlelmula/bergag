use crate::feeds::service;
use crate::security::redaction::redact;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const INTERVAL_SECONDS: u64 = 60;

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL_SECONDS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await;

        loop {
            interval.tick().await;
            let outcomes = {
                let state = app.state::<AppState>();
                service::refresh_due(state.inner()).await
            };
            match outcomes {
                Ok(outcomes) if !outcomes.is_empty() => {
                    if let Err(error) = app.emit("feeds:refreshed", outcomes) {
                        tracing::warn!(error = %redact(&error.to_string()), "feed refresh event failed");
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(error = %redact(&error.to_string()), "scheduled feed refresh failed");
                }
            }
        }
    });
}
