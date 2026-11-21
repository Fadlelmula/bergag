use super::command_error;
use crate::ai::{models, service};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn ai_settings_get(state: State<'_, AppState>) -> Result<models::AiSettingsDto, String> {
    service::get_settings(state.inner()).map_err(command_error)
}

#[tauri::command]
pub fn ai_settings_update(
    settings: models::AiSettingsDto,
    state: State<'_, AppState>,
) -> Result<models::AiSettingsDto, String> {
    service::save_settings(state.inner(), settings).map_err(command_error)
}

#[tauri::command]
pub fn ai_providers_list(state: State<'_, AppState>) -> Result<Vec<models::AiProviderDto>, String> {
    service::list_providers(state.inner()).map_err(command_error)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn ai_provider_save(
    id: Option<String>,
    name: String,
    kind: String,
    endpoint: String,
    model: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<models::AiProviderDto, String> {
    service::save_provider(
        state.inner(),
        id.as_deref(),
        &name,
        &kind,
        &endpoint,
        &model,
        enabled,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn ai_provider_delete(provider_id: String, state: State<'_, AppState>) -> Result<(), String> {
    service::delete_provider(state.inner(), &provider_id).map_err(command_error)
}

#[tauri::command]
pub fn ai_set_api_key(
    provider_id: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    service::set_api_key(state.inner(), &provider_id, &api_key).map_err(command_error)
}

#[tauri::command]
pub fn ai_clear_api_key(provider_id: String, state: State<'_, AppState>) -> Result<(), String> {
    service::clear_api_key(state.inner(), &provider_id).map_err(command_error)
}

#[tauri::command]
pub async fn ai_provider_test(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<models::AiTestResultDto, String> {
    service::test_provider(state.inner(), &provider_id)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn ai_summarize(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<models::AiCompletionDto, String> {
    service::run_feature(state.inner(), models::AiFeature::Summarize, &item_id, None)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn ai_suggest_labels(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<models::AiCompletionDto, String> {
    service::run_feature(
        state.inner(),
        models::AiFeature::SuggestLabels,
        &item_id,
        None,
    )
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn ai_ask(
    item_id: String,
    question: String,
    state: State<'_, AppState>,
) -> Result<models::AiCompletionDto, String> {
    service::run_feature(
        state.inner(),
        models::AiFeature::Ask,
        &item_id,
        Some(&question),
    )
    .await
    .map_err(command_error)
}

#[tauri::command]
pub fn ai_usage_stats(
    days: Option<i64>,
    state: State<'_, AppState>,
) -> Result<models::AiUsageStatsDto, String> {
    service::usage_stats(state.inner(), days.unwrap_or(30)).map_err(command_error)
}

#[tauri::command]
pub fn ai_usage_events(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<models::AiUsageEventDto>, String> {
    service::list_usage(state.inner(), limit.unwrap_or(50)).map_err(command_error)
}

#[tauri::command]
pub fn ai_usage_clear(state: State<'_, AppState>) -> Result<i64, String> {
    service::clear_usage(state.inner()).map_err(command_error)
}
