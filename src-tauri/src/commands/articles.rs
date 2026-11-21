use super::{active_profile_id, command_error, locked_connection};
use crate::articles::{models, service, store};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

const LIST_MODES: &[&str] = &[
    "all",
    "unread",
    "saved",
    "favorites",
    "archive",
    "history",
    "queue",
    "highlights",
    "collection",
];

fn validate_mode(mode: Option<String>) -> Result<String, String> {
    let mode = mode.unwrap_or_else(|| "unread".to_string());
    if LIST_MODES.contains(&mode.as_str()) {
        Ok(mode)
    } else {
        Err(command_error(AppError::Validation(
            "Unknown article list mode.".to_string(),
        )))
    }
}

#[tauri::command]
pub fn articles_list(
    mode: Option<String>,
    collection_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::ArticleListItemDto>, String> {
    let mode = validate_mode(mode)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(100).clamp(1, 200);
    store::list_articles(
        &connection,
        &profile_id,
        &mode,
        collection_id.as_deref(),
        limit,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn articles_get(
    id: String,
    state: State<'_, AppState>,
) -> Result<models::ArticleDetailDto, String> {
    service::detail(state.inner(), &id).map_err(command_error)
}

#[tauri::command]
pub fn articles_open(id: String, state: State<'_, AppState>) -> Result<(), String> {
    service::open(state.inner(), &id).map_err(command_error)
}

#[tauri::command]
pub async fn articles_extract(
    id: String,
    state: State<'_, AppState>,
) -> Result<models::ArticleDetailDto, String> {
    service::extract(state.inner(), &id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub fn articles_set_state(
    id: String,
    patch: models::ArticleStatePatch,
    state: State<'_, AppState>,
) -> Result<models::ArticleStateDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::update_state(&mut connection, &profile_id, &id, &patch).map_err(command_error)
}

#[tauri::command]
pub fn articles_bulk_update(
    ids: Vec<String>,
    patch: models::ArticleStatePatch,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::bulk_update_state(&mut connection, &profile_id, &ids, &patch).map_err(command_error)
}

#[tauri::command]
pub fn articles_history(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::HistoryEntryDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(100).clamp(1, 200);
    store::list_history(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn articles_highlights(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::HighlightEntryDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let limit = limit.unwrap_or(200).clamp(1, 500);
    store::list_highlights(&connection, &profile_id, limit).map_err(command_error)
}

#[tauri::command]
pub fn articles_add_note(
    id: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<models::ArticleNoteDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::add_note(&connection, &profile_id, &id, &body).map_err(command_error)
}

#[tauri::command]
pub fn articles_update_note(
    note_id: String,
    body: String,
    state: State<'_, AppState>,
) -> Result<models::ArticleNoteDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::update_note(&connection, &profile_id, &note_id, &body).map_err(command_error)
}

#[tauri::command]
pub fn articles_delete_note(note_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_note(&connection, &profile_id, &note_id).map_err(command_error)
}

#[tauri::command]
pub fn articles_add_highlight(
    id: String,
    text: String,
    start_offset: i64,
    end_offset: i64,
    state: State<'_, AppState>,
) -> Result<models::ArticleHighlightDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::add_highlight(
        &connection,
        &profile_id,
        &id,
        &text,
        start_offset,
        end_offset,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn articles_delete_highlight(
    highlight_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_highlight(&connection, &profile_id, &highlight_id).map_err(command_error)
}

#[tauri::command]
pub fn articles_list_collections(
    state: State<'_, AppState>,
) -> Result<Vec<models::CollectionDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_collections(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn articles_create_collection(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::CollectionDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::create_collection(&connection, &profile_id, &name, description.as_deref())
        .map_err(command_error)
}

#[tauri::command]
pub fn articles_delete_collection(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_collection(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn articles_add_to_collection(
    collection_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::add_to_collection(&connection, &profile_id, &collection_id, &item_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn articles_remove_from_collection(
    collection_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::remove_from_collection(&connection, &profile_id, &collection_id, &item_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn articles_cache_stats(
    state: State<'_, AppState>,
) -> Result<models::ArticleCacheStatsDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::cache_stats(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn articles_clear_cache(
    item_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    store::clear_cache(&mut connection, &profile_id, item_id.as_deref()).map_err(command_error)
}
