use super::models::{
    AiCompletionDto, AiFeature, AiProviderDto, AiProviderKind, AiRecordUsage, AiSettingsDto,
    AiUsageEventDto, AiUsageStatsDto,
};
use super::providers::validate_provider_config;
use crate::analytics::models::NamedCountDto;
use crate::credentials;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use crate::validation::normalize_display_name;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};

pub const AI_SETTINGS_KEY: &str = "ai.settings";
const MAX_PROVIDERS: i64 = 8;
const MAX_USAGE_EVENTS: i64 = 2_000;

#[derive(Debug, Clone)]
pub struct AiArticleContext {
    pub title: String,
    pub feed_title: String,
    pub content_text: String,
    pub highlights: Vec<String>,
    pub notes: Vec<String>,
    pub labels: Vec<String>,
}

pub fn get_settings(connection: &Connection, profile_id: &str) -> AppResult<AiSettingsDto> {
    let Some(value) = db::settings::get_profile_setting(connection, profile_id, AI_SETTINGS_KEY)?
    else {
        return Ok(AiSettingsDto::default());
    };
    serde_json::from_value(value)
        .map_err(|_| AppError::Internal("Stored AI settings are invalid.".to_string()))
        .map(AiSettingsDto::validated)?
}

pub fn save_settings(
    connection: &Connection,
    profile_id: &str,
    settings: AiSettingsDto,
) -> AppResult<AiSettingsDto> {
    let settings = settings.validated()?;
    if let Some(provider_id) = settings.provider_id.as_deref() {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM ai_providers WHERE profile_id = ?1 AND id = ?2)",
            params![profile_id, provider_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(AppError::Validation(
                "Choose an AI provider from this profile.".to_string(),
            ));
        }
    }
    let value = serde_json::to_value(&settings)?;
    db::settings::set_profile_setting(connection, profile_id, AI_SETTINGS_KEY, &value)?;
    Ok(settings)
}

fn parse_kind(raw: &str) -> rusqlite::Result<AiProviderKind> {
    AiProviderKind::parse(raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn provider_from_row(row: &rusqlite::Row<'_>, profile_id: &str) -> rusqlite::Result<AiProviderDto> {
    let kind: String = row.get("kind")?;
    let id: String = row.get("id")?;
    let kind = parse_kind(&kind)?;
    Ok(AiProviderDto {
        has_key: credentials::has_api_key(profile_id, &id),
        requires_key: kind.requires_key(),
        id,
        name: row.get("name")?,
        kind,
        endpoint: row.get("endpoint")?,
        model: row.get("model")?,
        enabled: row.get::<_, i64>("enabled")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_providers(connection: &Connection, profile_id: &str) -> AppResult<Vec<AiProviderDto>> {
    let mut statement = connection.prepare(
        "SELECT id, name, kind, endpoint, model, enabled, created_at, updated_at
         FROM ai_providers WHERE profile_id = ?1 ORDER BY name_normalized, created_at",
    )?;
    let rows = statement.query_map([profile_id], |row| provider_from_row(row, profile_id))?;
    rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        .map_err(Into::into)
}

pub fn get_provider(
    connection: &Connection,
    profile_id: &str,
    provider_id: &str,
) -> AppResult<AiProviderDto> {
    connection
        .query_row(
            "SELECT id, name, kind, endpoint, model, enabled, created_at, updated_at
             FROM ai_providers WHERE profile_id = ?1 AND id = ?2",
            params![profile_id, provider_id],
            |row| provider_from_row(row, profile_id),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("AI provider not found.".to_string()))
}

#[allow(clippy::too_many_arguments)]
pub fn save_provider(
    connection: &mut Connection,
    profile_id: &str,
    id: Option<&str>,
    name: &str,
    kind: AiProviderKind,
    endpoint: &str,
    model: &str,
    enabled: bool,
    local_providers_enabled: bool,
) -> AppResult<AiProviderDto> {
    validate_provider_config(kind, endpoint, model, local_providers_enabled)?;
    let (name, name_normalized) = normalize_display_name(name, "Provider name")?;
    let endpoint = endpoint.trim().to_string();
    let model = model.trim().to_string();
    let now = Utc::now().to_rfc3339();

    let transaction = connection.transaction()?;
    let provider_id = if let Some(existing_id) = id {
        let stored_profile: Option<String> = transaction
            .query_row(
                "SELECT profile_id FROM ai_providers WHERE id = ?1",
                [existing_id],
                |row| row.get(0),
            )
            .optional()?;
        match stored_profile {
            None => return Err(AppError::NotFound("AI provider not found.".to_string())),
            Some(stored_profile) if stored_profile != profile_id => {
                return Err(AppError::NotFound("AI provider not found.".to_string()))
            }
            Some(_) => {
                let duplicate: bool = transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM ai_providers
                     WHERE profile_id = ?1 AND name_normalized = ?2 AND id <> ?3)",
                    params![profile_id, name_normalized, existing_id],
                    |row| row.get(0),
                )?;
                if duplicate {
                    return Err(AppError::Conflict(
                        "A provider with that name already exists.".to_string(),
                    ));
                }
                transaction.execute(
                    "UPDATE ai_providers SET name = ?1, name_normalized = ?2, kind = ?3,
                     endpoint = ?4, model = ?5, enabled = ?6, updated_at = ?7
                     WHERE profile_id = ?8 AND id = ?9",
                    params![
                        name,
                        name_normalized,
                        kind.as_str(),
                        endpoint,
                        model,
                        i64::from(enabled),
                        now,
                        profile_id,
                        existing_id,
                    ],
                )?;
                existing_id.to_string()
            }
        }
    } else {
        let count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM ai_providers WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )?;
        if count >= MAX_PROVIDERS {
            return Err(AppError::Validation(format!(
                "This profile can have at most {MAX_PROVIDERS} AI providers."
            )));
        }
        let duplicate: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM ai_providers WHERE profile_id = ?1 AND name_normalized = ?2)",
            params![profile_id, name_normalized],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(AppError::Conflict(
                "A provider with that name already exists.".to_string(),
            ));
        }
        let provider_id = uuid::Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO ai_providers
             (id, profile_id, name, name_normalized, kind, endpoint, model, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                provider_id,
                profile_id,
                name,
                name_normalized,
                kind.as_str(),
                endpoint,
                model,
                i64::from(enabled),
                now,
            ],
        )?;
        provider_id
    };
    transaction.commit()?;
    get_provider(connection, profile_id, &provider_id)
}

pub fn delete_provider(
    connection: &mut Connection,
    profile_id: &str,
    provider_id: &str,
) -> AppResult<()> {
    let provider = get_provider(connection, profile_id, provider_id)?;
    credentials::delete_api_key(profile_id, provider_id)?;
    let transaction = connection.transaction()?;
    transaction.execute(
        "UPDATE ai_usage_events SET provider_id = NULL
         WHERE profile_id = ?1 AND provider_id = ?2",
        params![profile_id, provider_id],
    )?;
    transaction.execute(
        "DELETE FROM ai_providers WHERE profile_id = ?1 AND id = ?2",
        params![profile_id, provider_id],
    )?;
    transaction.commit()?;
    let mut settings = get_settings(connection, profile_id)?;
    if settings.provider_id.as_deref() == Some(provider_id) {
        settings.provider_id = None;
        settings.enabled = false;
        save_settings(connection, profile_id, settings)?;
    }
    let _ = provider;
    Ok(())
}

pub fn record_usage(
    connection: &Connection,
    profile_id: &str,
    usage: AiRecordUsage,
) -> AppResult<()> {
    let error = usage.error.map(|message| {
        let trimmed = message.chars().take(500).collect::<String>();
        redact(&trimmed)
    });
    connection.execute(
        "INSERT INTO ai_usage_events
         (id, profile_id, provider_id, provider_name, provider_kind, feature, success,
          input_chars, output_chars, prompt_tokens, completion_tokens, latency_ms, error, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            uuid::Uuid::new_v4().to_string(),
            profile_id,
            usage.provider_id,
            usage.provider_name,
            usage.provider_kind.as_str(),
            usage.feature.as_str(),
            i64::from(usage.success),
            usage.input_chars,
            usage.output_chars,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.latency_ms,
            error,
            Utc::now().to_rfc3339(),
        ],
    )?;
    connection.execute(
        "DELETE FROM ai_usage_events WHERE profile_id = ?1 AND id NOT IN
         (SELECT id FROM ai_usage_events WHERE profile_id = ?1 ORDER BY created_at DESC LIMIT ?2)",
        params![profile_id, MAX_USAGE_EVENTS],
    )?;
    Ok(())
}

fn usage_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiUsageEventDto> {
    let kind: String = row.get("provider_kind")?;
    let success: i64 = row.get("success")?;
    Ok(AiUsageEventDto {
        id: row.get("id")?,
        provider_id: row.get("provider_id")?,
        provider_name: row.get("provider_name")?,
        provider_kind: parse_kind(&kind)?,
        feature: row.get("feature")?,
        success: success != 0,
        input_chars: row.get("input_chars")?,
        output_chars: row.get("output_chars")?,
        prompt_tokens: row.get("prompt_tokens")?,
        completion_tokens: row.get("completion_tokens")?,
        latency_ms: row.get("latency_ms")?,
        error: row.get("error")?,
        created_at: row.get("created_at")?,
    })
}

pub fn list_usage(
    connection: &Connection,
    profile_id: &str,
    limit: i64,
) -> AppResult<Vec<AiUsageEventDto>> {
    let mut statement = connection.prepare(
        "SELECT id, provider_id, provider_name, provider_kind, feature, success,
                input_chars, output_chars, prompt_tokens, completion_tokens, latency_ms,
                error, created_at
         FROM ai_usage_events WHERE profile_id = ?1 ORDER BY created_at DESC LIMIT ?2",
    )?;
    let rows = statement.query_map(params![profile_id, limit], usage_event_from_row)?;
    rows.collect::<Result<Vec<_>, rusqlite::Error>>()
        .map_err(Into::into)
}

pub fn usage_stats(
    connection: &Connection,
    profile_id: &str,
    days: i64,
) -> AppResult<AiUsageStatsDto> {
    let cutoff = (Utc::now() - Duration::days(days)).to_rfc3339();
    let totals: (i64, i64, i64, i64, i64, i64, i64, Option<String>) = connection.query_row(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(input_chars), 0),
                COALESCE(SUM(output_chars), 0),
                COALESCE(SUM(prompt_tokens), 0),
                COALESCE(SUM(completion_tokens), 0),
                MAX(created_at)
         FROM ai_usage_events WHERE profile_id = ?1 AND created_at >= ?2",
        params![profile_id, cutoff],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        },
    )?;

    let mut statement = connection.prepare(
        "SELECT feature AS id, feature AS name, COUNT(*) AS count, NULL AS kind
         FROM ai_usage_events WHERE profile_id = ?1 AND created_at >= ?2
         GROUP BY feature ORDER BY count DESC, feature",
    )?;
    let by_feature = statement
        .query_map(params![profile_id, cutoff], |row| {
            Ok(NamedCountDto {
                id: row.get(0)?,
                name: row.get(1)?,
                count: row.get(2)?,
                kind: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    let mut statement = connection.prepare(
        "SELECT COALESCE(provider_id, 'deleted') AS id, provider_name AS name,
                COUNT(*) AS count, provider_kind AS kind
         FROM ai_usage_events WHERE profile_id = ?1 AND created_at >= ?2
         GROUP BY id, provider_name, provider_kind ORDER BY count DESC, provider_name",
    )?;
    let by_provider = statement
        .query_map(params![profile_id, cutoff], |row| {
            Ok(NamedCountDto {
                id: row.get(0)?,
                name: row.get(1)?,
                count: row.get(2)?,
                kind: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(AiUsageStatsDto {
        total_requests: totals.0,
        successful_requests: totals.1,
        failed_requests: totals.2,
        total_input_chars: totals.3,
        total_output_chars: totals.4,
        prompt_tokens: totals.5,
        completion_tokens: totals.6,
        last_used_at: totals.7,
        by_feature,
        by_provider,
    })
}

pub fn clear_usage(connection: &Connection, profile_id: &str) -> AppResult<i64> {
    let removed = connection.execute(
        "DELETE FROM ai_usage_events WHERE profile_id = ?1",
        [profile_id],
    )?;
    Ok(removed as i64)
}

pub fn article_context(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    include_highlights: bool,
    include_notes: bool,
    include_labels: bool,
) -> AppResult<AiArticleContext> {
    let (title, feed_title, content_text): (String, String, String) = connection
        .query_row(
            "SELECT i.title, COALESCE(f.title, ''),
                COALESCE(NULLIF(s.content_text, ''), COALESCE(i.summary, ''))
         FROM feed_items i
         LEFT JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
         LEFT JOIN article_sources s ON s.item_id = i.id AND s.profile_id = i.profile_id
         WHERE i.profile_id = ?1 AND i.id = ?2",
            params![profile_id, item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Article not found.".to_string()))?;

    let mut highlights = Vec::new();
    if include_highlights {
        let mut statement = connection.prepare(
            "SELECT text FROM article_highlights
             WHERE profile_id = ?1 AND item_id = ?2 ORDER BY created_at DESC LIMIT 20",
        )?;
        highlights = statement
            .query_map(params![profile_id, item_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
    }

    let mut notes = Vec::new();
    if include_notes {
        let mut statement = connection.prepare(
            "SELECT body FROM article_notes
             WHERE profile_id = ?1 AND item_id = ?2 ORDER BY created_at DESC LIMIT 20",
        )?;
        notes = statement
            .query_map(params![profile_id, item_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
    }

    let mut labels = Vec::new();
    if include_labels {
        let mut statement = connection.prepare(
            "SELECT l.name FROM labels l
             JOIN label_assignments a ON a.label_id = l.id AND a.profile_id = l.profile_id
             WHERE l.profile_id = ?1 AND a.item_id = ?2
             ORDER BY l.kind, l.name LIMIT 100",
        )?;
        labels = statement
            .query_map(params![profile_id, item_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
    }

    Ok(AiArticleContext {
        title,
        feed_title,
        content_text,
        highlights,
        notes,
        labels,
    })
}

pub fn available_labels(
    connection: &Connection,
    profile_id: &str,
    limit: i64,
) -> AppResult<Vec<String>> {
    let mut statement = connection
        .prepare("SELECT name FROM labels WHERE profile_id = ?1 ORDER BY kind, name LIMIT ?2")?;
    let rows = statement.query_map(params![profile_id, limit], |row| row.get(0))?;
    rows.collect::<Result<Vec<String>, rusqlite::Error>>()
        .map_err(Into::into)
}

pub fn completion_to_usage(
    completion: &AiCompletionDto,
    feature: AiFeature,
    provider_kind: AiProviderKind,
) -> AiRecordUsage {
    AiRecordUsage {
        provider_id: Some(completion.provider_id.clone()),
        provider_name: completion.provider_name.clone(),
        provider_kind,
        feature,
        success: true,
        input_chars: 0,
        output_chars: completion.text.chars().count() as i64,
        prompt_tokens: completion.prompt_tokens,
        completion_tokens: completion.completion_tokens,
        latency_ms: completion.latency_ms,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::models::{AiFeatureSettingsDto, AiPrivacySettingsDto};
    use crate::db::profiles;

    fn setup() -> (Connection, String, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        crate::db::migrations::run(&mut connection).unwrap();
        profiles::ensure_default_profile(&mut connection).unwrap();
        let first = db::settings::get_active_profile_id(&connection).unwrap();
        let second = profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    fn settings() -> AiSettingsDto {
        AiSettingsDto {
            enabled: true,
            provider_id: None,
            features: AiFeatureSettingsDto {
                summarize: true,
                suggest_labels: false,
                ask: false,
            },
            privacy: AiPrivacySettingsDto {
                allow_article_text: true,
                ..AiPrivacySettingsDto::default()
            },
        }
    }

    #[test]
    fn settings_default_to_disabled_and_round_trip() {
        let (connection, profile_id, _) = setup();
        assert!(!get_settings(&connection, &profile_id).unwrap().enabled);
        let saved = save_settings(&connection, &profile_id, settings()).unwrap();
        assert!(saved.enabled);
        let loaded = get_settings(&connection, &profile_id).unwrap();
        assert_eq!(loaded, saved);
    }

    #[test]
    fn providers_are_profile_scoped_and_unique() {
        let (mut connection, first, second) = setup();
        let provider = save_provider(
            &mut connection,
            &first,
            None,
            "Local",
            AiProviderKind::Ollama,
            "http://127.0.0.1:11434",
            "llama3.1",
            true,
            true,
        )
        .unwrap();
        assert_eq!(list_providers(&connection, &first).unwrap().len(), 1);
        assert!(list_providers(&connection, &second).unwrap().is_empty());
        assert!(get_provider(&connection, &second, &provider.id).is_err());

        let duplicate = save_provider(
            &mut connection,
            &first,
            None,
            " local ",
            AiProviderKind::Ollama,
            "http://127.0.0.1:11434",
            "llama3.1",
            true,
            true,
        );
        assert!(matches!(duplicate, Err(AppError::Conflict(_))));

        let updated = save_provider(
            &mut connection,
            &first,
            Some(&provider.id),
            "Local model",
            AiProviderKind::Ollama,
            "http://127.0.0.1:11435",
            "llama3.2",
            false,
            true,
        )
        .unwrap();
        assert_eq!(updated.name, "Local model");
        assert!(!updated.enabled);
    }

    #[test]
    fn deleting_provider_detaches_usage_and_clears_selection() {
        let (mut connection, profile_id, _) = setup();
        let provider = save_provider(
            &mut connection,
            &profile_id,
            None,
            "Local",
            AiProviderKind::Ollama,
            "http://127.0.0.1:11434",
            "llama3.1",
            true,
            true,
        )
        .unwrap();
        let mut stored = settings();
        stored.provider_id = Some(provider.id.clone());
        save_settings(&connection, &profile_id, stored).unwrap();
        record_usage(
            &connection,
            &profile_id,
            AiRecordUsage {
                provider_id: Some(provider.id.clone()),
                provider_name: provider.name.clone(),
                provider_kind: AiProviderKind::Ollama,
                feature: AiFeature::Summarize,
                success: true,
                input_chars: 100,
                output_chars: 50,
                prompt_tokens: 20,
                completion_tokens: 10,
                latency_ms: 250,
                error: None,
            },
        )
        .unwrap();

        delete_provider(&mut connection, &profile_id, &provider.id).unwrap();
        assert!(list_providers(&connection, &profile_id).unwrap().is_empty());
        let events = list_usage(&connection, &profile_id, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].provider_id.is_none());
        let loaded = get_settings(&connection, &profile_id).unwrap();
        assert!(!loaded.enabled);
        assert!(loaded.provider_id.is_none());
    }

    #[test]
    fn usage_stats_and_clear_are_profile_scoped() {
        let (connection, first, second) = setup();
        record_usage(
            &connection,
            &first,
            AiRecordUsage {
                provider_id: None,
                provider_name: "Local".to_string(),
                provider_kind: AiProviderKind::Ollama,
                feature: AiFeature::Summarize,
                success: true,
                input_chars: 120,
                output_chars: 40,
                prompt_tokens: 30,
                completion_tokens: 12,
                latency_ms: 300,
                error: Some("key=sk-secret123".to_string()),
            },
        )
        .unwrap();
        record_usage(
            &connection,
            &second,
            AiRecordUsage {
                provider_id: None,
                provider_name: "Other".to_string(),
                provider_kind: AiProviderKind::Ollama,
                feature: AiFeature::Ask,
                success: false,
                input_chars: 10,
                output_chars: 0,
                prompt_tokens: 0,
                completion_tokens: 0,
                latency_ms: 50,
                error: None,
            },
        )
        .unwrap();

        let stats = usage_stats(&connection, &first, 30).unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.prompt_tokens, 30);
        assert_eq!(stats.by_feature.len(), 1);
        let events = list_usage(&connection, &first, 10).unwrap();
        assert!(!events[0].error.clone().unwrap().contains("sk-secret123"));

        assert_eq!(clear_usage(&connection, &first).unwrap(), 1);
        assert_eq!(list_usage(&connection, &first, 10).unwrap().len(), 0);
        assert_eq!(list_usage(&connection, &second, 10).unwrap().len(), 1);
    }

    #[test]
    fn article_context_respects_privacy_flags() {
        let (connection, profile_id, _) = setup();
        connection.execute(
            "INSERT INTO feeds (id, profile_id, title, feed_url, feed_url_normalized, next_refresh_at)
             VALUES ('feed-1', ?1, 'Example', 'https://example.com/feed', 'https://example.com/feed', ?2)",
            params![profile_id, Utc::now().to_rfc3339()],
        ).unwrap();
        connection.execute(
            "INSERT INTO feed_items (id, profile_id, feed_id, guid, title, summary, discovered_at)
             VALUES ('item-1', ?1, 'feed-1', 'guid-1', 'Test article', 'Short summary', ?2)",
            params![profile_id, Utc::now().to_rfc3339()],
        ).unwrap();
        connection.execute(
            "INSERT INTO article_highlights (id, profile_id, item_id, text, start_offset, end_offset, created_at)
             VALUES ('h-1', ?1, 'item-1', 'Important line', 0, 1, ?2)",
            params![profile_id, Utc::now().to_rfc3339()],
        ).unwrap();
        connection
            .execute(
                "INSERT INTO article_notes (id, profile_id, item_id, body, created_at, updated_at)
             VALUES ('n-1', ?1, 'item-1', 'Private note', ?2, ?2)",
                params![profile_id, Utc::now().to_rfc3339()],
            )
            .unwrap();

        let private =
            article_context(&connection, &profile_id, "item-1", false, false, false).unwrap();
        assert_eq!(private.title, "Test article");
        assert_eq!(private.content_text, "Short summary");
        assert!(private.highlights.is_empty());
        assert!(private.notes.is_empty());

        let shared = article_context(&connection, &profile_id, "item-1", true, true, true).unwrap();
        assert_eq!(shared.highlights, vec!["Important line".to_string()]);
        assert_eq!(shared.notes, vec!["Private note".to_string()]);
    }
}
