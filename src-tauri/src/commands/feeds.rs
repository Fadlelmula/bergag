use super::{active_profile_id, command_error, locked_connection};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::feeds::{models, service};
use crate::state::AppState;
use crate::validation::{normalize_display_name, normalize_optional_text, normalize_trust_level};
use tauri::State;

fn validate_feed_update(
    title: &str,
    status: &str,
    interval: i64,
    category: Option<&str>,
    trust_level: &str,
) -> AppResult<(String, String, i64, Option<String>, String)> {
    let (title, _) = normalize_display_name(title, "Feed title")?;
    if title.chars().count() > 256 {
        return Err(AppError::Validation(
            "Feed title must be 256 characters or fewer.".to_string(),
        ));
    }
    if !matches!(status, "active" | "paused") {
        return Err(AppError::Validation(
            "Feed status must be active or paused.".to_string(),
        ));
    }
    let interval = interval.clamp(5, 10080);
    let category = normalize_optional_text(category, "Category", 64)?;
    let trust_level = normalize_trust_level(trust_level)?;
    Ok((title, status.to_string(), interval, category, trust_level))
}

#[tauri::command]
pub fn feeds_list_folders(
    state: State<'_, AppState>,
) -> Result<Vec<models::FeedFolderDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::list_folders(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn feeds_create_folder(
    name: String,
    state: State<'_, AppState>,
) -> Result<models::FeedFolderDto, String> {
    let (name, normalized) = normalize_display_name(&name, "Folder name").map_err(command_error)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::create_folder(
        &connection,
        &profile_id,
        &uuid::Uuid::new_v4().to_string(),
        &name,
        &normalized,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn feeds_delete_folder(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::delete_folder(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn feeds_list(state: State<'_, AppState>) -> Result<Vec<models::FeedDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::list_feeds(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn feeds_get(id: String, state: State<'_, AppState>) -> Result<models::FeedDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::get_feed(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub async fn feeds_preview(url: String) -> Result<models::FeedPreviewDto, String> {
    service::preview(&url).await.map_err(command_error)
}

#[tauri::command]
pub async fn feeds_discover(url: String) -> Result<Vec<models::DiscoveredFeedDto>, String> {
    service::discover(&url).await.map_err(command_error)
}

#[tauri::command]
pub async fn feeds_subscribe(
    url: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::SubscribeResultDto, String> {
    service::subscribe(state.inner(), &url, folder_id.as_deref(), None)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub fn feeds_update(
    id: String,
    title: String,
    folder_id: Option<String>,
    status: String,
    refresh_interval_minutes: i64,
    category: Option<String>,
    trust_level: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::FeedDto, String> {
    let trust_level = trust_level.unwrap_or_else(|| "untrusted".to_string());
    let (title, status, interval, category, trust_level) = validate_feed_update(
        &title,
        &status,
        refresh_interval_minutes,
        category.as_deref(),
        &trust_level,
    )
    .map_err(command_error)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::update_feed(
        &connection,
        &profile_id,
        &id,
        &title,
        folder_id.as_deref(),
        &status,
        interval,
        category.as_deref(),
        &trust_level,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn feeds_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    db::feeds::delete_feed(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub async fn feeds_refresh(
    feed_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<models::RefreshOutcomeDto>, String> {
    service::refresh(state.inner(), feed_id.as_deref())
        .await
        .map_err(command_error)
}

#[tauri::command]
pub fn feeds_logs(
    id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::RefreshLogDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(20).clamp(1, 100);
    db::feeds::list_refresh_logs(&connection, &profile_id, &id, limit).map_err(command_error)
}

#[tauri::command]
pub fn feeds_items(
    feed_id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::FeedItemDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(50).clamp(1, 200);
    db::feeds::recent_items(&connection, &profile_id, &feed_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn feeds_export_opml(state: State<'_, AppState>) -> Result<String, String> {
    service::export_opml(state.inner()).map_err(command_error)
}

#[tauri::command]
pub async fn feeds_import_opml(
    xml: String,
    state: State<'_, AppState>,
) -> Result<models::OpmlImportResultDto, String> {
    service::import_opml(state.inner(), &xml)
        .await
        .map_err(command_error)
}
