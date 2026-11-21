use super::{active_profile_id, command_error, locked_connection};
use crate::analytics::{models, store};
use crate::articles::models::ArticleListItemDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn analytics_reading(
    days: Option<i64>,
    state: State<'_, AppState>,
) -> Result<models::ReadingStatsDto, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::reading(&connection, &profile_id, days).map_err(command_error)
}

#[tauri::command]
pub fn analytics_library(state: State<'_, AppState>) -> Result<models::LibraryStatsDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::library(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn analytics_storage(state: State<'_, AppState>) -> Result<models::StorageStatsDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::storage(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn analytics_feed_health(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::FeedHealthDto>, String> {
    let limit = limit.unwrap_or(20).clamp(1, 50);
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::feed_health(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn analytics_unread_feeds(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::NamedCountDto>, String> {
    let limit = limit.unwrap_or(5).clamp(1, 20);
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::unread_feeds(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn analytics_top_labels(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::NamedCountDto>, String> {
    let limit = limit.unwrap_or(5).clamp(1, 20);
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::top_labels(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn analytics_articles(
    kind: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ArticleListItemDto>, String> {
    let limit = limit.unwrap_or(5).clamp(1, 20);
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::articles(&connection, &profile_id, &kind, limit).map_err(command_error)
}

#[tauri::command]
pub fn briefing_build(
    period: String,
    state: State<'_, AppState>,
) -> Result<models::BriefingDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::briefing(&connection, &profile_id, &period).map_err(command_error)
}
