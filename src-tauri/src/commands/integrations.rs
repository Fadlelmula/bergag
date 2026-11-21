use super::command_error;
use crate::feeds::models::RefreshOutcomeDto;
use crate::integrations::{models, registry, service};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn integrations_platforms_list() -> Vec<models::IntegrationPlatformDto> {
    registry::platforms()
}

#[tauri::command]
pub async fn integrations_preview(
    kind: String,
    input: String,
    state: State<'_, AppState>,
) -> Result<models::IntegrationPreviewResultDto, String> {
    service::preview(state.inner(), &kind, &input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn integrations_add(
    kind: String,
    input: String,
    category: Option<String>,
    trust_level: Option<String>,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::IntegrationSourceDto, String> {
    service::add_source(
        state.inner(),
        &kind,
        &input,
        category.as_deref(),
        trust_level.as_deref().unwrap_or("untrusted"),
        folder_id.as_deref(),
    )
    .await
    .map_err(command_error)
}

#[tauri::command]
pub fn integrations_list(
    state: State<'_, AppState>,
) -> Result<Vec<models::IntegrationSourceDto>, String> {
    service::list_sources(state.inner()).map_err(command_error)
}

#[tauri::command]
pub fn integrations_set_trust(
    source_id: String,
    trust_level: String,
    state: State<'_, AppState>,
) -> Result<models::IntegrationSourceDto, String> {
    service::set_trust(state.inner(), &source_id, &trust_level).map_err(command_error)
}

#[tauri::command]
pub async fn integrations_refresh(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<RefreshOutcomeDto>, String> {
    service::refresh_source(state.inner(), &source_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub fn integrations_delete(source_id: String, state: State<'_, AppState>) -> Result<(), String> {
    service::delete_source(state.inner(), &source_id).map_err(command_error)
}
