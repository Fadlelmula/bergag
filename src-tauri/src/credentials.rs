//! Narrow wrapper around the OS credential store.
//!
//! Secrets are never written to SQLite, settings, logs, diagnostics, or DTOs.
//! Keys are scoped by profile and provider id so deleting a profile or provider
//! cannot leave a credential usable by another profile.

use crate::error::{AppError, AppResult};
use keyring::Entry;

const AI_SERVICE: &str = "com.bergag.desktop.ai";
const MAX_SECRET_LENGTH: usize = 4096;

fn credential_user(profile_id: &str, provider_id: &str) -> String {
    format!("{profile_id}::{provider_id}")
}

fn entry(profile_id: &str, provider_id: &str) -> AppResult<Entry> {
    Entry::new(AI_SERVICE, &credential_user(profile_id, provider_id)).map_err(|_| {
        AppError::Internal("The operating system credential store is unavailable.".to_string())
    })
}

pub fn set_api_key(profile_id: &str, provider_id: &str, secret: &str) -> AppResult<()> {
    let secret = secret.trim();
    if secret.is_empty() || secret.chars().count() > MAX_SECRET_LENGTH {
        return Err(AppError::Validation(format!(
            "The API key must be between 1 and {MAX_SECRET_LENGTH} characters."
        )));
    }
    if secret.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "The API key cannot contain control characters.".to_string(),
        ));
    }
    entry(profile_id, provider_id)?
        .set_password(secret)
        .map_err(|_| {
            AppError::Internal(
                "The API key could not be saved in the credential store.".to_string(),
            )
        })
}

pub fn get_api_key(profile_id: &str, provider_id: &str) -> AppResult<String> {
    entry(profile_id, provider_id)?.get_password().map_err(|_| {
        AppError::Internal(
            "The saved API key could not be read from the credential store.".to_string(),
        )
    })
}

pub fn has_api_key(profile_id: &str, provider_id: &str) -> bool {
    entry(profile_id, provider_id)
        .and_then(|stored| {
            stored
                .get_password()
                .map_err(|_| AppError::Internal("Credential lookup failed.".to_string()))
        })
        .is_ok()
}

pub fn delete_api_key(profile_id: &str, provider_id: &str) -> AppResult<()> {
    match entry(profile_id, provider_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::error::Error::NoEntry) => Ok(()),
        Err(_) => Err(AppError::Internal(
            "The API key could not be removed from the credential store.".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_keys_are_scoped_by_profile_and_provider() {
        assert_eq!(
            credential_user("profile-a", "provider-1"),
            "profile-a::provider-1"
        );
        assert_ne!(
            credential_user("profile-a", "provider-1"),
            credential_user("profile-b", "provider-1")
        );
        assert_ne!(
            credential_user("profile-a", "provider-1"),
            credential_user("profile-a", "provider-2")
        );
    }

    #[test]
    fn rejects_empty_oversized_and_control_character_secrets() {
        assert!(set_api_key("profile", "provider", "   ").is_err());
        assert!(set_api_key("profile", "provider", &"x".repeat(4097)).is_err());
        assert!(set_api_key("profile", "provider", "abc\ndef").is_err());
    }
}
