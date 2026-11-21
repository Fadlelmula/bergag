use super::models::{IntegrationPreviewResultDto, IntegrationSourceDto};
use super::registry::{self, IntegrationKind};
use super::store::{self, NewSource};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::feeds::{models::RefreshOutcomeDto, network, service as feed_service};
use crate::security::redaction::redact;
use crate::state::AppState;
use crate::validation::{normalize_optional_text, normalize_trust_level};
use rusqlite::Connection;
use std::sync::MutexGuard;

fn locked(state: &AppState) -> AppResult<MutexGuard<'_, Connection>> {
    state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })
}

fn normalized_feed_url(feed_url: &str) -> AppResult<String> {
    let url = network::parse_public_url(feed_url)?;
    Ok(network::normalized_url(&url))
}

pub async fn preview(
    state: &AppState,
    raw_kind: &str,
    raw_input: &str,
) -> AppResult<IntegrationPreviewResultDto> {
    let kind = IntegrationKind::parse(raw_kind)?;
    let resolved = registry::resolve(kind, raw_input)?;
    let normalized = normalized_feed_url(&resolved.feed_url)?;
    let preview = feed_service::preview(&resolved.feed_url).await?;

    let (existing_source, duplicate_feed) = {
        let connection = locked(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let by_key = store::find_by_key(&connection, &profile_id, kind, &resolved.source_key)?;
        let by_url = store::find_by_feed_url(&connection, &profile_id, &normalized)?;
        let duplicate_feed =
            db::feeds::find_feed_by_normalized_url(&connection, &profile_id, &normalized)?
                .is_some();
        (by_key.or(by_url), duplicate_feed)
    };

    Ok(IntegrationPreviewResultDto {
        platform: kind.platform(),
        resolved_feed_url: resolved.feed_url,
        preview,
        duplicate_feed,
        duplicate_source: existing_source.is_some(),
        existing_source,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn add_source(
    state: &AppState,
    raw_kind: &str,
    raw_input: &str,
    category: Option<&str>,
    trust_level: &str,
    folder_id: Option<&str>,
) -> AppResult<IntegrationSourceDto> {
    let kind = IntegrationKind::parse(raw_kind)?;
    let resolved = registry::resolve(kind, raw_input)?;
    let normalized = normalized_feed_url(&resolved.feed_url)?;
    let category = normalize_optional_text(category, "Category", 64)?;
    let trust_level = normalize_trust_level(trust_level)?;
    let preview = feed_service::preview(&resolved.feed_url).await?;
    let title = normalize_optional_text(Some(&preview.title), "Source name", 256)?
        .or_else(|| resolved.suggested_title.clone())
        .ok_or_else(|| AppError::Validation("The feed did not provide a title.".to_string()))?;

    let (profile_id, existing_id) = {
        let connection = locked(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let by_key = store::find_by_key(&connection, &profile_id, kind, &resolved.source_key)?;
        let by_url = store::find_by_feed_url(&connection, &profile_id, &normalized)?;
        if let Some(url_source) = by_url.as_ref() {
            if by_key.as_ref().map(|source| source.id.as_str()) != Some(url_source.id.as_str()) {
                return Err(AppError::Conflict(format!(
                    "This feed is already connected as {}.",
                    url_source.display_name
                )));
            }
        }
        (profile_id, by_key.map(|source| source.id))
    };

    let subscription =
        feed_service::subscribe(state, &resolved.feed_url, folder_id, Some(&title)).await?;
    let feed_id = subscription.feed.id.clone();

    let connection = locked(state)?;
    let source = store::upsert_source(
        &connection,
        &profile_id,
        existing_id.as_deref(),
        &NewSource {
            kind,
            source_key: &resolved.source_key,
            display_name: &title,
            source_url: resolved.source_url.as_deref(),
            feed_url: &resolved.feed_url,
            feed_url_normalized: &normalized,
            category: category.as_deref(),
            trust_level: &trust_level,
            subscribed_feed_id: None,
        },
    )?;
    let source = store::set_subscription(&connection, &profile_id, &source.id, &feed_id)?;
    db::feeds::set_organization(
        &connection,
        &profile_id,
        &feed_id,
        category.as_deref(),
        &trust_level,
    )?;
    Ok(source)
}

pub fn list_sources(state: &AppState) -> AppResult<Vec<IntegrationSourceDto>> {
    let connection = locked(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::list_sources(&connection, &profile_id)
}

pub fn set_trust(
    state: &AppState,
    source_id: &str,
    trust_level: &str,
) -> AppResult<IntegrationSourceDto> {
    let trust_level = normalize_trust_level(trust_level)?;
    let connection = locked(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let current = store::get_source(&connection, &profile_id, source_id)?;
    let source = store::set_organization(
        &connection,
        &profile_id,
        source_id,
        current.category.as_deref(),
        &trust_level,
    )?;
    if let Some(feed_id) = source.feed_id.as_deref() {
        db::feeds::set_organization(
            &connection,
            &profile_id,
            feed_id,
            source.category.as_deref(),
            &trust_level,
        )?;
    }
    Ok(source)
}

pub async fn refresh_source(
    state: &AppState,
    source_id: &str,
) -> AppResult<Vec<RefreshOutcomeDto>> {
    let (profile_id, feed_id) = {
        let connection = locked(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let source = store::get_source(&connection, &profile_id, source_id)?;
        let feed_id = source.feed_id.ok_or_else(|| {
            AppError::Validation("This integration is not subscribed to a feed yet.".to_string())
        })?;
        (profile_id, feed_id)
    };

    let outcomes = feed_service::refresh(state, Some(&feed_id)).await?;
    let failure = outcomes
        .iter()
        .find(|outcome| outcome.status != "success" && outcome.status != "not_modified");
    let message = failure
        .and_then(|outcome| outcome.message.as_deref())
        .map(redact);

    let connection = locked(state)?;
    if let Some(message) = message {
        store::set_error(&connection, &profile_id, source_id, &message)?;
    } else {
        store::clear_error(&connection, &profile_id, source_id)?;
    }
    Ok(outcomes)
}

pub fn delete_source(state: &AppState, source_id: &str) -> AppResult<()> {
    let connection = locked(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::delete_source(&connection, &profile_id, source_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_platforms_are_rejected_before_network_access() {
        let kind = IntegrationKind::parse("bluesky").unwrap();
        assert!(registry::resolve(kind, "user.bsky.social").is_err());
        let monitor = IntegrationKind::parse("website_monitor").unwrap();
        assert!(registry::resolve(monitor, "https://example.com").is_err());
    }
}
