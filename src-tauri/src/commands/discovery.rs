use super::{active_profile_id, command_error, locked_connection};
use crate::articles::models::ArticleListItemDto;
use crate::articles::store as article_store;
use crate::discovery::{models, rules, service, store};
use crate::error::AppError;
use crate::state::AppState;
use crate::validation::normalize_optional_text;
use tauri::State;

const LABEL_KINDS: &[&str] = &["tag", "topic"];

fn validate_label_kind(kind: &str) -> Result<&str, String> {
    if LABEL_KINDS.contains(&kind) {
        Ok(kind)
    } else {
        Err(command_error(AppError::Validation(
            "Label kind must be tag or topic.".to_string(),
        )))
    }
}

fn validate_label_fields(
    color: Option<String>,
    description: Option<String>,
) -> Result<(Option<String>, Option<String>), String> {
    let color =
        normalize_optional_text(color.as_deref(), "Label color", 32).map_err(command_error)?;
    let description = normalize_optional_text(description.as_deref(), "Label description", 500)
        .map_err(command_error)?;
    Ok((color, description))
}

#[tauri::command]
pub fn discovery_list(
    state: State<'_, AppState>,
) -> Result<Vec<models::DiscoverySourceDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_sources(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub async fn discovery_preview(
    url: String,
    state: State<'_, AppState>,
) -> Result<models::DiscoveryPreviewResultDto, String> {
    service::preview(state.inner(), &url)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn discovery_add(
    url: String,
    category: Option<String>,
    trust_level: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::DiscoveryPreviewResultDto, String> {
    let trust_level = trust_level.unwrap_or_else(|| "untrusted".to_string());
    service::add_source(state.inner(), &url, category.as_deref(), &trust_level)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn discovery_subscribe(
    source_id: String,
    folder_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<crate::feeds::models::FeedDto, String> {
    service::subscribe_source(state.inner(), &source_id, folder_id.as_deref())
        .await
        .map_err(command_error)
}

#[tauri::command]
pub fn discovery_set_trust(
    source_id: String,
    trust_level: String,
    state: State<'_, AppState>,
) -> Result<models::DiscoverySourceDto, String> {
    service::set_source_trust(state.inner(), &source_id, &trust_level).map_err(command_error)
}

#[tauri::command]
pub fn discovery_delete(source_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_source(&connection, &profile_id, &source_id).map_err(command_error)
}

#[tauri::command]
pub fn labels_list(
    kind: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<models::LabelDto>, String> {
    let kind = match kind.as_deref() {
        Some(kind) => Some(validate_label_kind(kind)?),
        None => None,
    };
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_labels(&connection, &profile_id, kind).map_err(command_error)
}

#[tauri::command]
pub fn labels_create(
    kind: String,
    name: String,
    color: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::LabelDto, String> {
    let kind = validate_label_kind(&kind)?;
    let (color, description) = validate_label_fields(color, description)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    store::create_label(
        &connection,
        &profile_id,
        &id,
        kind,
        &name,
        color.as_deref(),
        description.as_deref(),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn labels_update(
    id: String,
    name: String,
    color: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<models::LabelDto, String> {
    let (color, description) = validate_label_fields(color, description)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::update_label(
        &connection,
        &profile_id,
        &id,
        &name,
        color.as_deref(),
        description.as_deref(),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn labels_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_label(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn labels_for_item(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::LabelDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_item_labels(&connection, &profile_id, &item_id).map_err(command_error)
}

#[tauri::command]
pub fn labels_items(
    label_id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ArticleListItemDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let ids = store::list_label_item_ids(
        &connection,
        &profile_id,
        &label_id,
        limit.unwrap_or(200).clamp(1, 200),
    )
    .map_err(command_error)?;
    article_store::list_articles_by_ids(
        &connection,
        &profile_id,
        &ids,
        limit.unwrap_or(200).clamp(1, 200),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn labels_assign(
    label_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::assign_label(&connection, &profile_id, &label_id, &item_id).map_err(command_error)
}

#[tauri::command]
pub fn labels_unassign(
    label_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::unassign_label(&connection, &profile_id, &label_id, &item_id).map_err(command_error)
}

#[tauri::command]
pub fn rules_list(state: State<'_, AppState>) -> Result<Vec<models::RuleDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_rules(&connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn rules_get(id: String, state: State<'_, AppState>) -> Result<models::RuleDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::get_rule(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn rules_create(
    name: String,
    description: Option<String>,
    rule_json: String,
    enabled: Option<bool>,
    is_smart_feed: Option<bool>,
    state: State<'_, AppState>,
) -> Result<models::RuleDto, String> {
    let description = normalize_optional_text(description.as_deref(), "Rule description", 500)
        .map_err(command_error)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::parse_and_validate_rule_json(&connection, &profile_id, &rule_json)
        .map_err(command_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    store::create_rule(
        &connection,
        &profile_id,
        &id,
        &name,
        description.as_deref(),
        &rule_json,
        enabled.unwrap_or(true),
        is_smart_feed.unwrap_or(false),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn rules_update(
    id: String,
    name: String,
    description: Option<String>,
    rule_json: String,
    enabled: bool,
    is_smart_feed: bool,
    state: State<'_, AppState>,
) -> Result<models::RuleDto, String> {
    let description = normalize_optional_text(description.as_deref(), "Rule description", 500)
        .map_err(command_error)?;
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::parse_and_validate_rule_json(&connection, &profile_id, &rule_json)
        .map_err(command_error)?;
    store::update_rule(
        &connection,
        &profile_id,
        &id,
        &name,
        description.as_deref(),
        &rule_json,
        enabled,
        is_smart_feed,
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn rules_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::delete_rule(&connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn rules_run(id: String, state: State<'_, AppState>) -> Result<models::RuleRunDto, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    rules::run_rule(&mut connection, &profile_id, &id).map_err(command_error)
}

#[tauri::command]
pub fn rules_run_enabled(state: State<'_, AppState>) -> Result<Vec<models::RuleRunDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let mut connection = locked_connection(&state).map_err(command_error)?;
    rules::run_enabled(&mut connection, &profile_id).map_err(command_error)
}

#[tauri::command]
pub fn rules_history(
    id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<models::RuleRunDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    store::list_rule_runs(
        &connection,
        &profile_id,
        &id,
        limit.unwrap_or(20).clamp(1, 100),
    )
    .map_err(command_error)
}

#[tauri::command]
pub fn rules_smart_feed(
    id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ArticleListItemDto>, String> {
    let profile_id = active_profile_id(&state).map_err(command_error)?;
    let connection = locked_connection(&state).map_err(command_error)?;
    let rule = store::get_rule(&connection, &profile_id, &id).map_err(command_error)?;
    if !rule.is_smart_feed {
        return Err(command_error(AppError::Validation(
            "That rule is not a smart feed.".to_string(),
        )));
    }
    let definition = store::parse_and_validate_rule_json(&connection, &profile_id, &rule.rule_json)
        .map_err(command_error)?;
    let ids = rules::matching_ids(&connection, &profile_id, &definition).map_err(command_error)?;
    article_store::list_articles_by_ids(
        &connection,
        &profile_id,
        &ids,
        limit.unwrap_or(100).clamp(1, 200),
    )
    .map_err(command_error)
}
