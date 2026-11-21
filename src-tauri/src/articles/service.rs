use super::content;
use super::extraction;
use super::models::ArticleDetailDto;
use super::store;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::feeds::network;
use crate::state::AppState;
use std::collections::HashMap;

fn setting_bool(
    connection: &rusqlite::Connection,
    profile_id: &str,
    key: &str,
    field: &str,
    default: bool,
) -> bool {
    db::settings::get_profile_setting(connection, profile_id, key)
        .ok()
        .flatten()
        .and_then(|value| value.get(field).and_then(|flag| flag.as_bool()))
        .unwrap_or(default)
}

pub fn open(state: &AppState, item_id: &str) -> AppResult<()> {
    let connection = state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let mark_read = setting_bool(&connection, &profile_id, "reading", "autoMarkRead", false);
    drop(connection);

    let mut connection = state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })?;
    store::record_open(&mut connection, &profile_id, item_id, mark_read)
}

pub async fn extract(state: &AppState, item_id: &str) -> AppResult<ArticleDetailDto> {
    let (profile_id, url) = {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let url = store::get_item_url(&connection, &profile_id, item_id)?;
        (profile_id, url)
    };

    let raw_url = url.ok_or_else(|| {
        AppError::Validation("This article has no source URL to extract.".to_string())
    })?;
    let parsed = network::parse_public_url(&raw_url)?;
    let result = network::fetch_bytes(&parsed, None, None, extraction::MAX_ARTICLE_BYTES).await?;
    if !result.status.is_success() {
        return Err(AppError::Validation(format!(
            "The article server returned HTTP {}.",
            result.status.as_u16()
        )));
    }

    let html = String::from_utf8_lossy(&result.bytes);
    let mut content = extraction::extract_from_html(&html, &parsed)?;

    let cache_images = {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        setting_bool(&connection, &profile_id, "offline", "cacheImages", true)
    };
    let images = if cache_images {
        extraction::cache_article_images(&content).await
    } else {
        Vec::new()
    };

    let mut data_urls = HashMap::new();
    for image in &images {
        data_urls.insert(
            image.url.clone(),
            extraction::data_url(&image.mime, &image.bytes),
        );
    }
    content.safe_html = content::rewrite_image_sources(&content.safe_html, &data_urls);
    content.image_urls = images.iter().map(|image| image.url.clone()).collect();
    let raw_html = html.chars().take(2_000_000).collect::<String>();

    {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        store::save_source(
            &connection,
            &profile_id,
            item_id,
            &store::StoredSource {
                raw_html: Some(&raw_html),
                content_text: &content.text,
                safe_html: &content.safe_html,
                word_count: content.word_count,
                image_urls: &content.image_urls,
                images: &images,
                source: "manual",
            },
        )?;
        store::get_detail(&connection, &profile_id, item_id)
    }
}

pub fn detail(state: &AppState, item_id: &str) -> AppResult<ArticleDetailDto> {
    let connection = state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::get_detail(&connection, &profile_id, item_id)
}
