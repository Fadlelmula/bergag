use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const ENDPOINT: Option<&str> = option_env!("BERGAG_UPDATE_ENDPOINT");
const PUBLIC_KEY: Option<&str> = option_env!("BERGAG_UPDATE_PUBLIC_KEY");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub configured: bool,
    pub signature_verification: bool,
    pub endpoint_host: Option<String>,
    pub current_version: String,
    pub check_available: bool,
    pub install_available: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub available: bool,
    pub version: Option<String>,
    pub checked_at: String,
    pub detail: String,
}

fn is_public_https(value: &str) -> bool {
    match Url::parse(value) {
        Ok(url) => url.scheme() == "https" && url.host_str().is_some(),
        Err(_) => false,
    }
}

fn has_valid_public_key(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 64 && !trimmed.contains(char::is_whitespace)
}

pub fn configured() -> bool {
    matches!((ENDPOINT, PUBLIC_KEY), (Some(endpoint), Some(key)) if is_public_https(endpoint) && has_valid_public_key(key))
}

pub fn status() -> UpdateStatus {
    let configured = configured();
    UpdateStatus {
        configured,
        signature_verification: configured,
        endpoint_host: if configured {
            ENDPOINT
                .and_then(|value| Url::parse(value).ok())
                .and_then(|url| url.host_str().map(str::to_string))
        } else {
            None
        },
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        check_available: configured,
        install_available: false,
        detail: if configured {
            "A signed release endpoint is compiled into this build; checks use HTTPS and require a valid minisign signature. Installation stays manual until release permission is granted."
                .to_string()
        } else {
            "No signed update endpoint or verification key is configured. This build has not checked for updates and cannot install one.".to_string()
        },
    }
}

fn endpoint() -> AppResult<Url> {
    let raw = ENDPOINT.ok_or_else(|| {
        AppError::Validation("No signed update endpoint is configured.".to_string())
    })?;
    Url::parse(raw).map_err(|_| {
        AppError::Validation("The configured update endpoint is not a valid URL.".to_string())
    })
}

pub async fn check<R: tauri::Runtime>(app: &AppHandle<R>) -> AppResult<UpdateCheck> {
    if !configured() {
        return Err(AppError::Validation(status().detail));
    }
    let endpoint = endpoint()?;
    if !is_public_https(endpoint.as_str()) {
        return Err(AppError::Validation(
            "Signed update checks require a public HTTPS endpoint.".to_string(),
        ));
    }
    let public_key = PUBLIC_KEY.expect("configured update key");

    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(20))
        .endpoints(vec![endpoint])
        .map_err(|error| AppError::Internal(format!("Could not configure update check: {error}")))?
        .pubkey(public_key)
        .build()
        .map_err(|error| {
            AppError::Internal(format!(
                "Could not prepare update check: {}",
                redact(&error.to_string())
            ))
        })?;

    let result = updater.check().await.map_err(|error| {
        AppError::Internal(format!(
            "Update check failed: {}",
            redact(&error.to_string())
        ))
    })?;
    let checked_at = Utc::now().to_rfc3339();
    match result {
        Some(update) => Ok(UpdateCheck {
            available: true,
            version: Some(update.version),
            checked_at,
            detail: "A signed update was found. Installation is disabled in local builds."
                .to_string(),
        }),
        None => Ok(UpdateCheck {
            available: false,
            version: None,
            checked_at,
            detail: "No signed update was available at the configured endpoint.".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_endpoint_must_be_public_https() {
        assert!(is_public_https("https://updates.example.com/latest.json"));
        assert!(!is_public_https("http://updates.example.com/latest.json"));
        assert!(!is_public_https("file:///C:/updates/latest.json"));
        assert!(!is_public_https("not a url"));
    }

    #[test]
    fn public_key_must_be_non_whitespace_and_long_enough() {
        assert!(has_valid_public_key(&"A".repeat(64)));
        assert!(!has_valid_public_key("short"));
        assert!(!has_valid_public_key(&format!("{} A", "A".repeat(63))));
    }

    #[test]
    fn status_never_claims_checking_or_installing_without_configuration() {
        let report = status();
        assert_eq!(report.check_available, report.configured);
        assert!(!report.install_available);
        assert_eq!(report.signature_verification, report.configured);
        if !report.configured {
            assert!(report.detail.contains("has not checked"));
        }
    }
}
