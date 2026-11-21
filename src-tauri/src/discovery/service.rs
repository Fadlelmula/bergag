use super::models::DiscoveryPreviewResultDto;
use super::store;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::feeds::{network, service as feed_service};
use crate::state::AppState;
use crate::validation::{normalize_optional_text, normalize_trust_level};
use rusqlite::Connection;

fn locked_connection(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
    state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })
}

pub async fn preview(state: &AppState, raw_url: &str) -> AppResult<DiscoveryPreviewResultDto> {
    let url = network::parse_public_url(raw_url)?;
    let normalized = network::normalized_url(&url);
    let preview = feed_service::preview(raw_url).await?;
    let connection = locked_connection(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let source = store::find_source_by_url(&connection, &profile_id, &normalized)?;
    Ok(DiscoveryPreviewResultDto { preview, source })
}

pub async fn add_source(
    state: &AppState,
    raw_url: &str,
    category: Option<&str>,
    trust_level: &str,
) -> AppResult<DiscoveryPreviewResultDto> {
    let url = network::parse_public_url(raw_url)?;
    let normalized = network::normalized_url(&url);
    let trust_level = normalize_trust_level(trust_level)?;
    let category = normalize_optional_text(category, "Category", 64)?;
    let preview = feed_service::preview(raw_url).await?;
    let title = normalize_optional_text(Some(&preview.title), "Source title", 256)?
        .ok_or_else(|| AppError::Validation("The feed did not provide a title.".to_string()))?;
    let description = normalize_optional_text(preview.description.as_deref(), "Description", 500)?;

    let connection = locked_connection(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let existing = store::find_source_by_url(&connection, &profile_id, &normalized)?;
    let id = existing
        .map(|source| source.id)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let source = store::upsert_source(
        &connection,
        &profile_id,
        &id,
        &store::NewSource {
            feed_url: &normalized,
            feed_url_normalized: &normalized,
            site_url: preview.site_url.as_deref(),
            title: &title,
            category: category.as_deref(),
            description: description.as_deref(),
            trust_level: &trust_level,
        },
    )?;
    drop(connection);
    Ok(DiscoveryPreviewResultDto {
        preview,
        source: Some(source),
    })
}

pub fn set_source_trust(
    state: &AppState,
    source_id: &str,
    trust_level: &str,
) -> AppResult<super::models::DiscoverySourceDto> {
    let trust_level = normalize_trust_level(trust_level)?;
    let connection = locked_connection(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let source = store::get_source(&connection, &profile_id, source_id)?;
    let updated = store::set_source_trust(&connection, &profile_id, source_id, &trust_level)?;
    if let Some(feed_id) = source.subscribed_feed_id.as_deref() {
        db::feeds::set_organization(
            &connection,
            &profile_id,
            feed_id,
            source.category.as_deref(),
            &trust_level,
        )?;
    }
    Ok(updated)
}

pub async fn subscribe_source(
    state: &AppState,
    source_id: &str,
    folder_id: Option<&str>,
) -> AppResult<crate::feeds::models::FeedDto> {
    let (profile_id, source) = {
        let connection = locked_connection(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let source = store::get_source(&connection, &profile_id, source_id)?;
        (profile_id, source)
    };
    let result = feed_service::subscribe(
        state,
        &source.feed_url,
        folder_id,
        Some(source.title.as_str()),
    )
    .await?;
    let feed_id = result.feed.id.clone();
    {
        let connection = locked_connection(state)?;
        store::set_source_subscription(&connection, &profile_id, source_id, Some(&feed_id))?;
        db::feeds::set_organization(
            &connection,
            &profile_id,
            &feed_id,
            source.category.as_deref(),
            &source.trust_level,
        )?;
    }
    Ok(result.feed)
}
