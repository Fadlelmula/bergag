use super::models::{DiscoverySourceDto, LabelDto, RuleCandidate, RuleDto, RuleRunDto};
use crate::error::{AppError, AppResult};
use crate::validation::{normalize_display_name, normalize_optional_text};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn required_id(connection: &Connection, table: &str, profile_id: &str, id: &str) -> AppResult<()> {
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1 AND profile_id = ?2)");
    let exists: bool = connection.query_row(&sql, params![id, profile_id], |row| row.get(0))?;
    if !exists {
        return Err(AppError::NotFound(
            "That item is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

fn map_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<DiscoverySourceDto> {
    Ok(DiscoverySourceDto {
        id: row.get(0)?,
        feed_url: row.get(1)?,
        site_url: row.get(2)?,
        title: row.get(3)?,
        category: row.get(4)?,
        description: row.get(5)?,
        trust_level: row.get(6)?,
        subscribed_feed_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

const SOURCE_COLUMNS: &str = "id, feed_url, site_url, title, category, description,
    trust_level, subscribed_feed_id, created_at, updated_at";

pub fn list_sources(
    connection: &Connection,
    profile_id: &str,
) -> AppResult<Vec<DiscoverySourceDto>> {
    let sql = format!(
        "SELECT {SOURCE_COLUMNS} FROM discovery_sources WHERE profile_id = ?1
         ORDER BY title COLLATE NOCASE ASC, feed_url ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map([profile_id], map_source)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub struct NewSource<'a> {
    pub feed_url: &'a str,
    pub feed_url_normalized: &'a str,
    pub site_url: Option<&'a str>,
    pub title: &'a str,
    pub category: Option<&'a str>,
    pub description: Option<&'a str>,
    pub trust_level: &'a str,
}

pub fn upsert_source(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    source: &NewSource<'_>,
) -> AppResult<DiscoverySourceDto> {
    let timestamp = now();
    connection.execute(
        "INSERT INTO discovery_sources (
            id, profile_id, feed_url, feed_url_normalized, site_url, title,
            category, description, trust_level, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
         ON CONFLICT(profile_id, feed_url_normalized) DO UPDATE SET
            feed_url = excluded.feed_url, site_url = excluded.site_url,
            title = excluded.title, category = excluded.category,
            description = excluded.description, trust_level = excluded.trust_level,
            updated_at = excluded.updated_at",
        params![
            id,
            profile_id,
            source.feed_url,
            source.feed_url_normalized,
            source.site_url,
            source.title,
            source.category,
            source.description,
            source.trust_level,
            timestamp,
        ],
    )?;
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM discovery_sources
                 WHERE profile_id = ?1 AND feed_url_normalized = ?2"
            ),
            params![profile_id, source.feed_url_normalized],
            map_source,
        )
        .map_err(AppError::from)
}

pub fn find_source_by_url(
    connection: &Connection,
    profile_id: &str,
    normalized_url: &str,
) -> AppResult<Option<DiscoverySourceDto>> {
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM discovery_sources
                 WHERE profile_id = ?1 AND feed_url_normalized = ?2"
            ),
            params![profile_id, normalized_url],
            map_source,
        )
        .optional()
        .map_err(AppError::from)
}

pub fn get_source(
    connection: &Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<DiscoverySourceDto> {
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM discovery_sources WHERE id = ?1 AND profile_id = ?2"
            ),
            params![id, profile_id],
            map_source,
        )
        .optional()?
        .ok_or_else(|| {
            AppError::NotFound("That discovery entry is not available in this profile.".to_string())
        })
}

pub fn set_source_trust(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    trust_level: &str,
) -> AppResult<DiscoverySourceDto> {
    let changed = connection.execute(
        "UPDATE discovery_sources SET trust_level = ?1, updated_at = ?2
         WHERE id = ?3 AND profile_id = ?4",
        params![trust_level, now(), id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That discovery entry is not available in this profile.".to_string(),
        ));
    }
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM discovery_sources WHERE id = ?1 AND profile_id = ?2"
            ),
            params![id, profile_id],
            map_source,
        )
        .map_err(AppError::from)
}

pub fn set_source_subscription(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
    feed_id: Option<&str>,
) -> AppResult<()> {
    let changed = connection.execute(
        "UPDATE discovery_sources SET subscribed_feed_id = ?1, updated_at = ?2
         WHERE id = ?3 AND profile_id = ?4
           AND (?1 IS NULL OR EXISTS (
             SELECT 1 FROM feeds WHERE id = ?1 AND profile_id = ?4
           ))",
        params![feed_id, now(), source_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::Validation(
            "That discovery entry or feed is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn delete_source(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM discovery_sources WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That discovery entry is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

fn map_label(row: &rusqlite::Row<'_>) -> rusqlite::Result<LabelDto> {
    Ok(LabelDto {
        id: row.get(0)?,
        kind: row.get(1)?,
        name: row.get(2)?,
        color: row.get(3)?,
        description: row.get(4)?,
        usage_count: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

const LABEL_COLUMNS: &str = "l.id, l.kind, l.name, l.color, l.description,
    (SELECT COUNT(*) FROM label_assignments a WHERE a.label_id = l.id AND a.profile_id = l.profile_id) AS usage_count,
    l.created_at, l.updated_at";

pub fn list_labels(
    connection: &Connection,
    profile_id: &str,
    kind: Option<&str>,
) -> AppResult<Vec<LabelDto>> {
    let sql = format!(
        "SELECT {LABEL_COLUMNS} FROM labels l
         WHERE l.profile_id = ?1 AND (?2 IS NULL OR l.kind = ?2)
         ORDER BY l.kind ASC, l.name COLLATE NOCASE ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map(params![profile_id, kind], map_label)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_label(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    kind: &str,
    name: &str,
    color: Option<&str>,
    description: Option<&str>,
) -> AppResult<LabelDto> {
    let (name, normalized) = normalize_display_name(name, "Label name")?;
    let timestamp = now();
    connection.execute(
        "INSERT INTO labels (
            id, profile_id, kind, name, name_normalized, color, description,
            created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            profile_id,
            kind,
            name,
            normalized,
            color,
            description,
            timestamp,
        ],
    )?;
    connection
        .query_row(
            &format!("SELECT {LABEL_COLUMNS} FROM labels l WHERE l.id = ?1 AND l.profile_id = ?2"),
            params![id, profile_id],
            map_label,
        )
        .map_err(AppError::from)
}

pub fn update_label(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    name: &str,
    color: Option<&str>,
    description: Option<&str>,
) -> AppResult<LabelDto> {
    let (name, normalized) = normalize_display_name(name, "Label name")?;
    let changed = connection.execute(
        "UPDATE labels SET name = ?1, name_normalized = ?2, color = ?3,
            description = ?4, updated_at = ?5
         WHERE id = ?6 AND profile_id = ?7",
        params![name, normalized, color, description, now(), id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That label is not available in this profile.".to_string(),
        ));
    }
    connection
        .query_row(
            &format!("SELECT {LABEL_COLUMNS} FROM labels l WHERE l.id = ?1 AND l.profile_id = ?2"),
            params![id, profile_id],
            map_label,
        )
        .map_err(AppError::from)
}

pub fn delete_label(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM labels WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That label is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn list_label_item_ids(
    connection: &Connection,
    profile_id: &str,
    label_id: &str,
    limit: usize,
) -> AppResult<Vec<String>> {
    required_id(connection, "labels", profile_id, label_id)?;
    let mut statement = connection.prepare(
        "SELECT item_id FROM label_assignments
         WHERE profile_id = ?1 AND label_id = ?2
         ORDER BY created_at DESC LIMIT ?3",
    )?;
    let rows = statement
        .query_map(params![profile_id, label_id, limit as i64], |row| {
            row.get(0)
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn assign_label(
    connection: &Connection,
    profile_id: &str,
    label_id: &str,
    item_id: &str,
) -> AppResult<()> {
    required_id(connection, "labels", profile_id, label_id)?;
    required_id(connection, "feed_items", profile_id, item_id)?;
    connection.execute(
        "INSERT OR IGNORE INTO label_assignments (profile_id, label_id, item_id, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![profile_id, label_id, item_id, now()],
    )?;
    Ok(())
}

pub fn unassign_label(
    connection: &Connection,
    profile_id: &str,
    label_id: &str,
    item_id: &str,
) -> AppResult<()> {
    connection.execute(
        "DELETE FROM label_assignments
         WHERE profile_id = ?1 AND label_id = ?2 AND item_id = ?3",
        params![profile_id, label_id, item_id],
    )?;
    Ok(())
}

pub fn list_item_labels(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
) -> AppResult<Vec<LabelDto>> {
    let sql = format!(
        "SELECT {LABEL_COLUMNS} FROM labels l
         JOIN label_assignments a ON a.label_id = l.id AND a.profile_id = l.profile_id
         WHERE l.profile_id = ?1 AND a.item_id = ?2
         ORDER BY l.kind ASC, l.name COLLATE NOCASE ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map(params![profile_id, item_id], map_label)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn map_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<RuleDto> {
    Ok(RuleDto {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        rule_json: row.get(3)?,
        enabled: row.get::<_, i64>(4)? != 0,
        is_smart_feed: row.get::<_, i64>(5)? != 0,
        last_run_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const RULE_COLUMNS: &str = "id, name, description, rule_json, enabled, is_smart_feed,
    last_run_at, created_at, updated_at";

pub fn list_rules(connection: &Connection, profile_id: &str) -> AppResult<Vec<RuleDto>> {
    let sql = format!(
        "SELECT {RULE_COLUMNS} FROM rules WHERE profile_id = ?1
         ORDER BY is_smart_feed ASC, name COLLATE NOCASE ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map([profile_id], map_rule)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_rule(connection: &Connection, profile_id: &str, id: &str) -> AppResult<RuleDto> {
    connection
        .query_row(
            &format!("SELECT {RULE_COLUMNS} FROM rules WHERE id = ?1 AND profile_id = ?2"),
            params![id, profile_id],
            map_rule,
        )
        .optional()?
        .ok_or_else(|| {
            AppError::NotFound("That rule is not available in this profile.".to_string())
        })
}

pub fn create_rule(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    name: &str,
    description: Option<&str>,
    rule_json: &str,
    enabled: bool,
    is_smart_feed: bool,
) -> AppResult<RuleDto> {
    let (name, normalized) = normalize_display_name(name, "Rule name")?;
    let timestamp = now();
    connection.execute(
        "INSERT INTO rules (
            id, profile_id, name, name_normalized, description, rule_json,
            enabled, is_smart_feed, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            profile_id,
            name,
            normalized,
            description,
            rule_json,
            enabled as i64,
            is_smart_feed as i64,
            timestamp,
        ],
    )?;
    get_rule(connection, profile_id, id)
}

pub fn update_rule(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    name: &str,
    description: Option<&str>,
    rule_json: &str,
    enabled: bool,
    is_smart_feed: bool,
) -> AppResult<RuleDto> {
    let (name, normalized) = normalize_display_name(name, "Rule name")?;
    let changed = connection.execute(
        "UPDATE rules SET name = ?1, name_normalized = ?2, description = ?3,
            rule_json = ?4, enabled = ?5, is_smart_feed = ?6, updated_at = ?7
         WHERE id = ?8 AND profile_id = ?9",
        params![
            name,
            normalized,
            description,
            rule_json,
            enabled as i64,
            is_smart_feed as i64,
            now(),
            id,
            profile_id,
        ],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That rule is not available in this profile.".to_string(),
        ));
    }
    get_rule(connection, profile_id, id)
}

pub fn delete_rule(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM rules WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That rule is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn mark_rule_run(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    connection.execute(
        "UPDATE rules SET last_run_at = ?1 WHERE id = ?2 AND profile_id = ?3",
        params![now(), id, profile_id],
    )?;
    Ok(())
}

pub fn record_run(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    rule_id: &str,
    started_at: &str,
    status: &str,
    matched_count: i64,
    changed_count: i64,
    message: Option<&str>,
) -> AppResult<RuleRunDto> {
    let finished_at = now();
    connection.execute(
        "INSERT INTO rule_runs (
            id, profile_id, rule_id, started_at, finished_at, status,
            matched_count, changed_count, message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            profile_id,
            rule_id,
            started_at,
            finished_at,
            status,
            matched_count,
            changed_count,
            message,
        ],
    )?;
    Ok(RuleRunDto {
        id: id.to_string(),
        rule_id: rule_id.to_string(),
        started_at: started_at.to_string(),
        finished_at,
        status: status.to_string(),
        matched_count,
        changed_count,
        message: message.map(str::to_string),
    })
}

pub fn list_rule_runs(
    connection: &Connection,
    profile_id: &str,
    rule_id: &str,
    limit: usize,
) -> AppResult<Vec<RuleRunDto>> {
    let mut statement = connection.prepare(
        "SELECT id, rule_id, started_at, finished_at, status, matched_count,
            changed_count, message
         FROM rule_runs
         WHERE profile_id = ?1 AND rule_id = ?2
         ORDER BY started_at DESC LIMIT ?3",
    )?;
    let rows = statement
        .query_map(params![profile_id, rule_id, limit as i64], |row| {
            Ok(RuleRunDto {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                status: row.get(4)?,
                matched_count: row.get(5)?,
                changed_count: row.get(6)?,
                message: row.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn rule_candidates(connection: &Connection, profile_id: &str) -> AppResult<Vec<RuleCandidate>> {
    let mut statement = connection.prepare(
        "SELECT i.id, i.title, COALESCE(i.summary, ''), COALESCE(src.content_text, ''),
            f.id, f.title, COALESCE(f.category, ''), f.trust_level,
            COALESCE(s.is_read, 0), COALESCE(s.is_saved, 0), COALESCE(s.is_favorite, 0),
            COALESCE(s.is_archived, 0), COALESCE(s.is_queued, 0)
         FROM feed_items i
         JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
         LEFT JOIN article_sources src ON src.item_id = i.id AND src.profile_id = i.profile_id
         LEFT JOIN article_states s ON s.item_id = i.id AND s.profile_id = i.profile_id
         WHERE i.profile_id = ?1
         ORDER BY COALESCE(i.published_at, i.discovered_at) DESC
         LIMIT 5000",
    )?;
    let candidates = statement
        .query_map([profile_id], |row| {
            Ok(RuleCandidate {
                id: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
                content_text: row.get(3)?,
                feed_id: row.get(4)?,
                feed_title: row.get(5)?,
                category: row.get(6)?,
                trust_level: row.get(7)?,
                is_read: row.get::<_, i64>(8)? != 0,
                is_saved: row.get::<_, i64>(9)? != 0,
                is_favorite: row.get::<_, i64>(10)? != 0,
                is_archived: row.get::<_, i64>(11)? != 0,
                is_queued: row.get::<_, i64>(12)? != 0,
                label_ids: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut labels = connection
        .prepare("SELECT item_id, label_id FROM label_assignments WHERE profile_id = ?1")?;
    let mut by_item: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for row in labels.query_map([profile_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (item_id, label_id) = row?;
        by_item.entry(item_id).or_default().push(label_id);
    }
    let mut candidates = candidates;
    for candidate in &mut candidates {
        if let Some(labels) = by_item.remove(&candidate.id) {
            candidate.label_ids = labels;
        }
    }
    Ok(candidates)
}

pub fn validate_label_references(
    connection: &Connection,
    profile_id: &str,
    definition: &super::models::RuleDefinition,
) -> AppResult<()> {
    use super::models::{RuleAction, RuleCondition};

    for condition in &definition.conditions {
        match condition {
            RuleCondition::HasLabel { label_id } => {
                required_id(connection, "labels", profile_id, label_id)?;
            }
            RuleCondition::FeedIs { feed_id } => {
                required_id(connection, "feeds", profile_id, feed_id)?;
            }
            _ => {}
        }
    }
    for action in &definition.actions {
        match action {
            RuleAction::AddLabel { label_id } | RuleAction::RemoveLabel { label_id } => {
                required_id(connection, "labels", profile_id, label_id)?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn parse_and_validate_rule_json(
    connection: &Connection,
    profile_id: &str,
    rule_json: &str,
) -> AppResult<super::models::RuleDefinition> {
    if rule_json.len() > 20_000 {
        return Err(AppError::Validation(
            "Rule definition must be 20,000 bytes or fewer.".to_string(),
        ));
    }
    let definition: super::models::RuleDefinition =
        serde_json::from_str(rule_json).map_err(|_| {
            AppError::Validation(
                "Rule definition is not valid. Rebuild it with the visual editor.".to_string(),
            )
        })?;
    if definition.conditions.is_empty() || definition.conditions.len() > 20 {
        return Err(AppError::Validation(
            "A rule needs between 1 and 20 conditions.".to_string(),
        ));
    }
    if definition.actions.is_empty() || definition.actions.len() > 20 {
        return Err(AppError::Validation(
            "A rule needs between 1 and 20 actions.".to_string(),
        ));
    }
    for condition in &definition.conditions {
        match condition {
            super::models::RuleCondition::TitleContains { value }
            | super::models::RuleCondition::SummaryContains { value }
            | super::models::RuleCondition::ContentContains { value }
            | super::models::RuleCondition::FeedTitleContains { value } => {
                if value.trim().is_empty() || value.chars().count() > 200 {
                    return Err(AppError::Validation(
                        "Rule text conditions must be between 1 and 200 characters.".to_string(),
                    ));
                }
            }
            super::models::RuleCondition::CategoryIs { value } => {
                normalize_optional_text(Some(value), "Category", 64)?.ok_or_else(|| {
                    AppError::Validation("A category condition cannot be empty.".to_string())
                })?;
            }
            super::models::RuleCondition::TrustAtLeast { value } => {
                crate::validation::normalize_trust_level(value)?;
            }
            super::models::RuleCondition::HasLabel { label_id } => {
                if label_id.trim().is_empty() {
                    return Err(AppError::Validation("Choose a label.".to_string()));
                }
            }
            super::models::RuleCondition::FeedIs { feed_id } => {
                if feed_id.trim().is_empty() {
                    return Err(AppError::Validation("Choose a feed.".to_string()));
                }
            }
            _ => {}
        }
    }
    for action in &definition.actions {
        if let super::models::RuleAction::AddLabel { label_id }
        | super::models::RuleAction::RemoveLabel { label_id } = action
        {
            if label_id.trim().is_empty() {
                return Err(AppError::Validation("Choose a label.".to_string()));
            }
        }
    }
    validate_label_references(connection, profile_id, &definition)?;
    Ok(definition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, feeds};

    fn setup() -> (rusqlite::Connection, String, String) {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        db::profiles::ensure_default_profile(&mut connection).unwrap();
        let first = db::settings::get_active_profile_id(&connection).unwrap();
        let second = db::profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    fn new_feed(url: &str) -> feeds::NewFeed<'_> {
        feeds::NewFeed {
            title: "Example feed",
            site_url: None,
            feed_url: url,
            feed_url_normalized: url,
            feed_type: "rss",
            description: None,
            language: None,
            etag: None,
            last_modified: None,
            folder_id: None,
        }
    }

    fn seed_item(connection: &mut rusqlite::Connection, profile_id: &str, feed_id: &str) -> String {
        feeds::subscribe(
            connection,
            profile_id,
            feed_id,
            &new_feed("https://example.com/feed.xml"),
            &[],
        )
        .unwrap();
        let item = feeds::NewItem {
            guid: "item-1",
            title: "Rust security update",
            url: Some("https://example.com/post"),
            author: None,
            summary: Some("A release"),
            content_html: None,
            published_at: None,
        };
        feeds::insert_items(connection, profile_id, feed_id, &[item]).unwrap();
        connection
            .query_row(
                "SELECT id FROM feed_items WHERE profile_id = ?1 AND feed_id = ?2",
                params![profile_id, feed_id],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn discovery_sources_are_profile_scoped_and_upserted() {
        let (connection, first, second) = setup();
        let source = NewSource {
            feed_url: "https://example.com/feed.xml",
            feed_url_normalized: "https://example.com/feed.xml",
            site_url: Some("https://example.com"),
            title: "Example",
            category: Some("News"),
            description: None,
            trust_level: "medium",
        };
        let created = upsert_source(&connection, &first, "source-1", &source).unwrap();
        assert_eq!(created.trust_level, "medium");
        assert_eq!(list_sources(&connection, &first).unwrap().len(), 1);
        assert!(list_sources(&connection, &second).unwrap().is_empty());
        assert!(get_source(&connection, &second, "source-1").is_err());

        let updated = NewSource {
            title: "Example updated",
            category: Some("Security"),
            trust_level: "high",
            ..source
        };
        let stored = upsert_source(&connection, &first, "source-2", &updated).unwrap();
        assert_eq!(stored.id, "source-1");
        assert_eq!(stored.title, "Example updated");
        assert_eq!(stored.trust_level, "high");
        assert_eq!(list_sources(&connection, &first).unwrap().len(), 1);
    }

    #[test]
    fn labels_are_profile_scoped_and_count_assignments() {
        let (mut connection, first, second) = setup();
        let item_id = seed_item(&mut connection, &first, "feed-1");
        let label = create_label(
            &connection,
            &first,
            "label-1",
            "tag",
            "Security",
            None,
            None,
        )
        .unwrap();
        assert_eq!(label.usage_count, 0);
        assign_label(&connection, &first, "label-1", &item_id).unwrap();
        let labels = list_item_labels(&connection, &first, &item_id).unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "Security");
        assert_eq!(
            list_labels(&connection, &first, None).unwrap()[0].usage_count,
            1
        );
        assert!(list_labels(&connection, &second, None).unwrap().is_empty());
        assert!(assign_label(&connection, &second, "label-1", &item_id).is_err());
        assert!(list_label_item_ids(&connection, &second, "label-1", 10).is_err());
    }

    #[test]
    fn rule_json_requires_same_profile_references() {
        let (mut connection, first, second) = setup();
        seed_item(&mut connection, &first, "feed-1");
        create_label(&connection, &first, "label-1", "topic", "Rust", None, None).unwrap();

        let valid = r#"{"matchMode":"all","conditions":[{"field":"feedIs","feedId":"feed-1"},{"field":"hasLabel","labelId":"label-1"}],"actions":[{"action":"addLabel","labelId":"label-1"}]}"#;
        assert!(parse_and_validate_rule_json(&connection, &first, valid).is_ok());
        assert!(parse_and_validate_rule_json(&connection, &second, valid).is_err());

        let unknown_feed = r#"{"matchMode":"any","conditions":[{"field":"feedIs","feedId":"missing"}],"actions":[{"action":"markRead","value":true}]}"#;
        assert!(parse_and_validate_rule_json(&connection, &first, unknown_feed).is_err());

        let too_large = format!(
            r#"{{"matchMode":"all","conditions":[{{"field":"titleContains","value":"{}"}}],"actions":[{{"action":"markRead","value":true}}]}}"#,
            "x".repeat(300)
        );
        assert!(parse_and_validate_rule_json(&connection, &first, &too_large).is_err());
    }
}
