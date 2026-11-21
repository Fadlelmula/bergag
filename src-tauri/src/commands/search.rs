use super::{active_profile_id, command_error, locked_connection};
use crate::error::AppError;
use crate::search::{models, query, store, stories};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn search_query(
    query_text: String,
    scope: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::SearchResultDto>, String> {
    let scope = scope.unwrap_or_else(|| "all".to_string());
    let plan = query::parse(&query_text, &scope).map_err(command_error)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(100).clamp(1, 200);
    store::search(&connection, &profile_id, &plan, limit).map_err(command_error)
}

#[tauri::command]
pub fn search_stats(state: State<'_, AppState>) -> Result<models::SearchStatsDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::stats(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn search_presets_list(
    state: State<'_, AppState>,
) -> Result<Vec<models::SearchPresetDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_presets(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn search_presets_save(
    id: Option<String>,
    name: String,
    query_text: String,
    scope: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::SearchPresetDto, String> {
    let scope = scope.unwrap_or_else(|| "all".to_string());
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::save_preset(
        &connection,
        &profile_id,
        id.as_deref(),
        &name,
        &query_text,
        &scope,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn search_presets_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_preset(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn stories_list(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::StoryGroupDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(50).clamp(1, 200);
    stories::list_stories(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn stories_get(
    id: String,
    state: State<'_, AppState>,
) -> Result<models::StoryGroupDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    stories::get_story(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn stories_timeline(
    days: Option<i64>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::StoryTimelineBucketDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let days = days.unwrap_or(30).clamp(1, 365);
    let limit = limit.unwrap_or(100).clamp(1, 500);
    stories::timeline(&connection, &profile_id, days, limit).map_err(command_error)
}

#[tauri::command]
pub fn stories_detect(state: State<'_, AppState>) -> Result<models::StoryDetectionDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    stories::detect_duplicates(&mut connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn stories_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    stories::delete_story(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn articles_related(
    id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::RelatedArticleDto>, String> {
    if id.trim().is_empty() {
        return Err(command_error(AppError::Validation(
            "Choose an article.".to_string(),
        )));
    }
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(5).clamp(1, 20);
    stories::related_articles(&connection, &profile_id, &id, limit).map_err(command_error)
}
