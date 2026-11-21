use super::models::{
    AiCompletionDto, AiFeature, AiProviderDto, AiProviderKind, AiRecordUsage, AiSettingsDto,
    AiTestResultDto,
};
use super::{providers, store};
use crate::credentials;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::validation::normalize_optional_text;
use rusqlite::Connection;
use std::sync::MutexGuard;
use std::time::Instant;

const MAX_ARTICLE_CHARS: usize = 12_000;
const MAX_QUESTION_CHARS: usize = 1_000;

fn lock(state: &AppState) -> AppResult<MutexGuard<'_, Connection>> {
    state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })
}

fn truncate(value: &str, maximum: usize) -> String {
    if value.chars().count() <= maximum {
        return value.to_string();
    }
    value.chars().take(maximum).collect()
}

struct Prepared {
    profile_id: String,
    provider: AiProviderDto,
    secret: Option<String>,
    system: String,
    prompt: String,
    input_chars: i64,
}

fn ensure_provider_ready(
    settings: &AiSettingsDto,
    provider: &AiProviderDto,
    feature: AiFeature,
) -> AppResult<()> {
    if !settings.enabled {
        return Err(AppError::Validation(
            "Enable AI in Settings → AI before using it.".to_string(),
        ));
    }
    if !provider.enabled {
        return Err(AppError::Validation(
            "This AI provider is disabled.".to_string(),
        ));
    }
    if provider.kind.is_local() && !settings.privacy.local_providers_enabled {
        return Err(AppError::Validation(
            "Enable local AI providers in Settings → AI.".to_string(),
        ));
    }
    if feature == AiFeature::ConnectionTest {
        return Ok(());
    }
    if settings.provider_id.as_deref() != Some(provider.id.as_str()) {
        return Err(AppError::Validation(
            "Select this provider in Settings → AI before using it.".to_string(),
        ));
    }
    let feature_enabled = match feature {
        AiFeature::Summarize => settings.features.summarize,
        AiFeature::SuggestLabels => settings.features.suggest_labels,
        AiFeature::Ask => settings.features.ask,
        AiFeature::ConnectionTest => true,
    };
    if !feature_enabled {
        return Err(AppError::Validation(
            "This AI feature is turned off in Settings → AI.".to_string(),
        ));
    }
    if !settings.privacy.allow_article_text {
        return Err(AppError::Validation(
            "Allow article text in Settings → AI before using article features.".to_string(),
        ));
    }
    Ok(())
}

fn build_feature_prompt(
    connection: &Connection,
    profile_id: &str,
    feature: AiFeature,
    item_id: &str,
    question: Option<&str>,
    settings: &AiSettingsDto,
) -> AppResult<(String, String)> {
    let context = store::article_context(
        connection,
        profile_id,
        item_id,
        settings.privacy.allow_highlights,
        settings.privacy.allow_notes,
        settings.privacy.allow_labels,
    )?;

    let system = match feature {
        AiFeature::Summarize => {
            "Summarize the supplied article in at most five short plain-text bullet lines. Do not invent facts."
                .to_string()
        }
        AiFeature::SuggestLabels => {
            "Choose the best matching labels from the supplied allowlist. Reply with at most five names, one per line, using only exact names from the list. Do not invent labels."
                .to_string()
        }
        AiFeature::Ask => {
            "Answer the user's question using only the supplied local article context. If the answer is not present, say so briefly."
                .to_string()
        }
        AiFeature::ConnectionTest => "Reply with OK.".to_string(),
    };

    let mut sections = vec![
        format!("Title: {}", context.title),
        format!("Feed: {}", context.feed_title),
        format!(
            "Article text:\n{}",
            truncate(&context.content_text, MAX_ARTICLE_CHARS)
        ),
    ];
    if settings.privacy.allow_highlights && !context.highlights.is_empty() {
        sections.push(format!(
            "Saved highlights:\n{}",
            context.highlights.join("\n- ")
        ));
    }
    if settings.privacy.allow_notes && !context.notes.is_empty() {
        sections.push(format!("Private notes:\n{}", context.notes.join("\n- ")));
    }
    if settings.privacy.allow_labels && !context.labels.is_empty() {
        sections.push(format!(
            "Labels already on this article:\n{}",
            context.labels.join("\n- ")
        ));
    }
    if feature == AiFeature::SuggestLabels {
        if !settings.privacy.allow_labels {
            return Err(AppError::Validation(
                "Allow label names in Settings → AI before requesting label suggestions."
                    .to_string(),
            ));
        }
        let labels = store::available_labels(connection, profile_id, 100)?;
        if labels.is_empty() {
            return Err(AppError::Validation(
                "Create a tag or topic before requesting AI label suggestions.".to_string(),
            ));
        }
        sections.push(format!(
            "Allowed label names:\n{}",
            labels
                .iter()
                .map(|name| format!("- {name}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if feature == AiFeature::Ask {
        let question = normalize_optional_text(question, "Question", MAX_QUESTION_CHARS)?
            .ok_or_else(|| AppError::Validation("Enter a question.".to_string()))?;
        sections.push(format!("Question:\n{question}"));
    }

    Ok((system, sections.join("\n\n")))
}

fn prepare(
    state: &AppState,
    provider_id: &str,
    feature: AiFeature,
    item_id: Option<&str>,
    question: Option<&str>,
) -> AppResult<Prepared> {
    let (profile_id, provider, system, prompt, available_labels) = {
        let connection = lock(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let settings = store::get_settings(&connection, &profile_id)?;
        let provider = store::get_provider(&connection, &profile_id, provider_id)?;
        ensure_provider_ready(&settings, &provider, feature)?;
        let (system, prompt) = if feature == AiFeature::ConnectionTest {
            ("Reply with OK.".to_string(), "Reply with OK.".to_string())
        } else {
            build_feature_prompt(
                &connection,
                &profile_id,
                feature,
                item_id.ok_or_else(|| {
                    AppError::Validation("Choose an article for this AI feature.".to_string())
                })?,
                question,
                &settings,
            )?
        };
        let available_labels = if feature == AiFeature::SuggestLabels {
            store::available_labels(&connection, &profile_id, 100)?
        } else {
            Vec::new()
        };
        (profile_id, provider, system, prompt, available_labels)
    };

    if feature == AiFeature::SuggestLabels && available_labels.is_empty() {
        return Err(AppError::Validation(
            "Create a tag or topic before requesting AI label suggestions.".to_string(),
        ));
    }

    let secret = if provider.requires_key {
        Some(credentials::get_api_key(&profile_id, &provider.id)?)
    } else {
        None
    };
    let input_chars = (system.chars().count() + prompt.chars().count()) as i64;

    Ok(Prepared {
        profile_id,
        provider,
        secret,
        system,
        prompt,
        input_chars,
    })
}

fn record_failure(
    state: &AppState,
    prepared: &Prepared,
    feature: AiFeature,
    latency_ms: i64,
    error: &str,
) {
    let Ok(connection) = lock(state) else {
        return;
    };
    let _ = store::record_usage(
        &connection,
        &prepared.profile_id,
        AiRecordUsage {
            provider_id: Some(prepared.provider.id.clone()),
            provider_name: prepared.provider.name.clone(),
            provider_kind: prepared.provider.kind,
            feature,
            success: false,
            input_chars: prepared.input_chars,
            output_chars: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
            latency_ms,
            error: Some(error.to_string()),
        },
    );
}

pub fn get_settings(state: &AppState) -> AppResult<AiSettingsDto> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::get_settings(&connection, &profile_id)
}

pub fn save_settings(state: &AppState, settings: AiSettingsDto) -> AppResult<AiSettingsDto> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::save_settings(&connection, &profile_id, settings)
}

pub fn list_providers(state: &AppState) -> AppResult<Vec<AiProviderDto>> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::list_providers(&connection, &profile_id)
}

#[allow(clippy::too_many_arguments)]
pub fn save_provider(
    state: &AppState,
    id: Option<&str>,
    name: &str,
    kind: &str,
    endpoint: &str,
    model: &str,
    enabled: bool,
) -> AppResult<AiProviderDto> {
    let mut connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let settings = store::get_settings(&connection, &profile_id)?;
    let kind = AiProviderKind::parse(kind)?;
    store::save_provider(
        &mut connection,
        &profile_id,
        id,
        name,
        kind,
        endpoint,
        model,
        enabled,
        settings.privacy.local_providers_enabled,
    )
}

pub fn delete_provider(state: &AppState, provider_id: &str) -> AppResult<()> {
    let mut connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::delete_provider(&mut connection, &profile_id, provider_id)
}

pub fn set_api_key(state: &AppState, provider_id: &str, api_key: &str) -> AppResult<()> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::get_provider(&connection, &profile_id, provider_id)?;
    drop(connection);
    credentials::set_api_key(&profile_id, provider_id, api_key)
}

pub fn clear_api_key(state: &AppState, provider_id: &str) -> AppResult<()> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::get_provider(&connection, &profile_id, provider_id)?;
    drop(connection);
    credentials::delete_api_key(&profile_id, provider_id)
}

pub async fn test_provider(state: &AppState, provider_id: &str) -> AppResult<AiTestResultDto> {
    let prepared = prepare(state, provider_id, AiFeature::ConnectionTest, None, None)?;
    let started = Instant::now();
    let result = providers::complete(providers::ProviderRequest {
        provider: &prepared.provider,
        secret: prepared.secret.as_deref(),
        system: &prepared.system,
        prompt: &prepared.prompt,
        max_tokens: 16,
    })
    .await;
    let latency_ms = started.elapsed().as_millis() as i64;

    match result {
        Ok(_) => {
            let Ok(connection) = lock(state) else {
                return Ok(AiTestResultDto {
                    ok: true,
                    message: "Provider responded successfully.".to_string(),
                    latency_ms,
                });
            };
            let _ = store::record_usage(
                &connection,
                &prepared.profile_id,
                AiRecordUsage {
                    provider_id: Some(prepared.provider.id.clone()),
                    provider_name: prepared.provider.name.clone(),
                    provider_kind: prepared.provider.kind,
                    feature: AiFeature::ConnectionTest,
                    success: true,
                    input_chars: prepared.input_chars,
                    output_chars: 2,
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    latency_ms,
                    error: None,
                },
            );
            Ok(AiTestResultDto {
                ok: true,
                message: "Provider responded successfully.".to_string(),
                latency_ms,
            })
        }
        Err(error) => {
            let message = error.user_message();
            record_failure(
                state,
                &prepared,
                AiFeature::ConnectionTest,
                latency_ms,
                &message,
            );
            Ok(AiTestResultDto {
                ok: false,
                message,
                latency_ms,
            })
        }
    }
}

pub async fn run_feature(
    state: &AppState,
    feature: AiFeature,
    item_id: &str,
    question: Option<&str>,
) -> AppResult<AiCompletionDto> {
    let provider_id = {
        let connection = lock(state)?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let settings = store::get_settings(&connection, &profile_id)?;
        settings.provider_id.clone().ok_or_else(|| {
            AppError::Validation("Select an AI provider in Settings → AI.".to_string())
        })?
    };
    let prepared = prepare(state, &provider_id, feature, Some(item_id), question)?;

    let started = Instant::now();
    let result = providers::complete(providers::ProviderRequest {
        provider: &prepared.provider,
        secret: prepared.secret.as_deref(),
        system: &prepared.system,
        prompt: &prepared.prompt,
        max_tokens: if feature == AiFeature::Summarize {
            700
        } else {
            1000
        },
    })
    .await;
    let latency_ms = started.elapsed().as_millis() as i64;

    match result {
        Ok(completion) => {
            let dto = AiCompletionDto {
                feature: feature.as_str().to_string(),
                text: completion.text,
                provider_id: prepared.provider.id.clone(),
                provider_name: prepared.provider.name.clone(),
                model: prepared.provider.model.clone(),
                prompt_tokens: completion.prompt_tokens,
                completion_tokens: completion.completion_tokens,
                latency_ms,
            };
            let mut usage = store::completion_to_usage(&dto, feature, prepared.provider.kind);
            usage.input_chars = prepared.input_chars;
            if let Ok(connection) = lock(state) {
                let _ = store::record_usage(&connection, &prepared.profile_id, usage);
            }
            Ok(dto)
        }
        Err(error) => {
            let message = error.user_message();
            record_failure(state, &prepared, feature, latency_ms, &message);
            Err(AppError::Internal(message))
        }
    }
}

pub fn usage_stats(state: &AppState, days: i64) -> AppResult<super::models::AiUsageStatsDto> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::usage_stats(&connection, &profile_id, days.clamp(1, 365))
}

pub fn list_usage(state: &AppState, limit: i64) -> AppResult<Vec<super::models::AiUsageEventDto>> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::list_usage(&connection, &profile_id, limit.clamp(1, 200))
}

pub fn clear_usage(state: &AppState) -> AppResult<i64> {
    let connection = lock(state)?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    store::clear_usage(&connection, &profile_id)
}
