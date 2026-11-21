use super::models::IntegrationSourceDto;
use super::registry::IntegrationKind;
use crate::error::{AppError, AppResult};
use chrono::Utc;
use rusqlite::{params, types::Type, Connection, OptionalExtension, Row};

pub struct NewSource<'a> {
    pub kind: IntegrationKind,
    pub source_key: &'a str,
    pub display_name: &'a str,
    pub source_url: Option<&'a str>,
    pub feed_url: &'a str,
    pub feed_url_normalized: &'a str,
    pub category: Option<&'a str>,
    pub trust_level: &'a str,
    pub subscribed_feed_id: Option<&'a str>,
}

fn parse_kind(raw: &str) -> rusqlite::Result<IntegrationKind> {
    IntegrationKind::parse(raw)
        .map_err(|error| rusqlite::Error::FromSqlConversionFailure(1, Type::Text, Box::new(error)))
}

fn source_from_row(row: &Row<'_>) -> rusqlite::Result<IntegrationSourceDto> {
    let kind = parse_kind(&row.get::<_, String>(1)?)?;
    Ok(IntegrationSourceDto {
        id: row.get(0)?,
        kind: kind.as_str().to_string(),
        source_key: row.get(2)?,
        display_name: row.get(3)?,
        source_url: row.get(4)?,
        feed_url: row.get(5)?,
        feed_id: row.get(6)?,
        category: row.get(7)?,
        trust_level: row.get(8)?,
        status: row.get(9)?,
        last_error: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SOURCE_COLUMNS: &str = "id, kind, source_key, display_name, source_url, feed_url,
     subscribed_feed_id, category, trust_level, status, last_error, created_at, updated_at";

pub fn list_sources(
    connection: &Connection,
    profile_id: &str,
) -> AppResult<Vec<IntegrationSourceDto>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {SOURCE_COLUMNS} FROM integration_sources
         WHERE profile_id = ?1 ORDER BY created_at DESC, display_name COLLATE NOCASE"
    ))?;
    let rows = statement.query_map([profile_id], source_from_row)?;
    Ok(rows.collect::<Result<Vec<_>, rusqlite::Error>>()?)
}

pub fn get_source(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
) -> AppResult<IntegrationSourceDto> {
    get_source_opt(connection, profile_id, source_id)?
        .ok_or_else(|| AppError::NotFound("That integration source was not found.".to_string()))
}

pub fn get_source_opt(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
) -> AppResult<Option<IntegrationSourceDto>> {
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM integration_sources
                 WHERE profile_id = ?1 AND id = ?2"
            ),
            params![profile_id, source_id],
            source_from_row,
        )
        .optional()
        .map_err(Into::into)
}

pub fn find_by_key(
    connection: &Connection,
    profile_id: &str,
    kind: IntegrationKind,
    source_key: &str,
) -> AppResult<Option<IntegrationSourceDto>> {
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM integration_sources
                 WHERE profile_id = ?1 AND kind = ?2 AND source_key = ?3"
            ),
            params![profile_id, kind.as_str(), source_key],
            source_from_row,
        )
        .optional()
        .map_err(Into::into)
}

pub fn find_by_feed_url(
    connection: &Connection,
    profile_id: &str,
    feed_url_normalized: &str,
) -> AppResult<Option<IntegrationSourceDto>> {
    connection
        .query_row(
            &format!(
                "SELECT {SOURCE_COLUMNS} FROM integration_sources
                 WHERE profile_id = ?1 AND feed_url_normalized = ?2"
            ),
            params![profile_id, feed_url_normalized],
            source_from_row,
        )
        .optional()
        .map_err(Into::into)
}

pub fn upsert_source(
    connection: &Connection,
    profile_id: &str,
    source_id: Option<&str>,
    source: &NewSource<'_>,
) -> AppResult<IntegrationSourceDto> {
    let by_key = find_by_key(connection, profile_id, source.kind, source.source_key)?;
    let by_url = find_by_feed_url(connection, profile_id, source.feed_url_normalized)?;
    if let Some(existing) = by_key.as_ref() {
        if source_id.is_some_and(|id| id != existing.id.as_str()) {
            return Err(AppError::Conflict(
                "That integration source is already connected.".to_string(),
            ));
        }
    }
    if let Some(existing) = by_url.as_ref() {
        if source_id.is_some_and(|id| id != existing.id.as_str()) {
            return Err(AppError::Conflict(format!(
                "This feed is already connected as {}.",
                existing.display_name
            )));
        }
    }
    if by_key.is_some()
        && by_url.is_some()
        && by_key.as_ref().unwrap().id.as_str() != by_url.as_ref().unwrap().id.as_str()
    {
        return Err(AppError::Conflict(
            "This integration conflicts with an existing source.".to_string(),
        ));
    }

    let id = source_id
        .map(str::to_string)
        .or_else(|| by_key.as_ref().map(|entry| entry.id.clone()))
        .or_else(|| by_url.as_ref().map(|entry| entry.id.clone()))
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();
    let exists = get_source_opt(connection, profile_id, &id)?.is_some();
    if exists {
        connection.execute(
            "UPDATE integration_sources SET kind = ?1, source_key = ?2, display_name = ?3,
             source_url = ?4, feed_url = ?5, feed_url_normalized = ?6, category = ?7,
             trust_level = ?8, subscribed_feed_id = ?9, status = 'active', last_error = NULL,
             updated_at = ?10 WHERE profile_id = ?11 AND id = ?12",
            params![
                source.kind.as_str(),
                source.source_key,
                source.display_name,
                source.source_url,
                source.feed_url,
                source.feed_url_normalized,
                source.category,
                source.trust_level,
                source.subscribed_feed_id,
                now,
                profile_id,
                id,
            ],
        )?;
    } else {
        connection.execute(
            "INSERT INTO integration_sources (
               id, profile_id, kind, source_key, display_name, source_url, feed_url,
               feed_url_normalized, category, trust_level, subscribed_feed_id, status,
               last_error, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', NULL, ?12, ?13)",
            params![
                id,
                profile_id,
                source.kind.as_str(),
                source.source_key,
                source.display_name,
                source.source_url,
                source.feed_url,
                source.feed_url_normalized,
                source.category,
                source.trust_level,
                source.subscribed_feed_id,
                now,
                now,
            ],
        )?;
    }
    get_source(connection, profile_id, &id)
}

pub fn set_organization(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
    category: Option<&str>,
    trust_level: &str,
) -> AppResult<IntegrationSourceDto> {
    let existing = get_source(connection, profile_id, source_id)?;
    connection.execute(
        "UPDATE integration_sources SET category = ?1, trust_level = ?2, updated_at = ?3
         WHERE profile_id = ?4 AND id = ?5",
        params![
            category,
            trust_level,
            Utc::now().to_rfc3339(),
            profile_id,
            source_id
        ],
    )?;
    get_source(connection, profile_id, &existing.id)
}

pub fn set_subscription(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
    feed_id: &str,
) -> AppResult<IntegrationSourceDto> {
    connection.execute(
        "UPDATE integration_sources SET subscribed_feed_id = ?1, status = 'active',
         last_error = NULL, updated_at = ?2 WHERE profile_id = ?3 AND id = ?4",
        params![feed_id, Utc::now().to_rfc3339(), profile_id, source_id],
    )?;
    get_source(connection, profile_id, source_id)
}

pub fn set_error(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
    message: &str,
) -> AppResult<IntegrationSourceDto> {
    connection.execute(
        "UPDATE integration_sources SET status = 'error', last_error = ?1, updated_at = ?2
         WHERE profile_id = ?3 AND id = ?4",
        params![message, Utc::now().to_rfc3339(), profile_id, source_id],
    )?;
    get_source(connection, profile_id, source_id)
}

pub fn clear_error(
    connection: &Connection,
    profile_id: &str,
    source_id: &str,
) -> AppResult<IntegrationSourceDto> {
    connection.execute(
        "UPDATE integration_sources SET status = 'active', last_error = NULL, updated_at = ?1
         WHERE profile_id = ?2 AND id = ?3 AND status <> 'active'",
        params![Utc::now().to_rfc3339(), profile_id, source_id],
    )?;
    get_source(connection, profile_id, source_id)
}

pub fn delete_source(connection: &Connection, profile_id: &str, source_id: &str) -> AppResult<()> {
    let deleted = connection.execute(
        "DELETE FROM integration_sources WHERE profile_id = ?1 AND id = ?2",
        params![profile_id, source_id],
    )?;
    if deleted == 0 {
        return Err(AppError::NotFound(
            "That integration source was not found.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, profiles};

    fn setup() -> (Connection, String, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        profiles::ensure_default_profile(&mut connection).unwrap();
        let first = db::settings::get_active_profile_id(&connection).unwrap();
        let second = profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    fn create_feed(connection: &Connection, profile_id: &str, id: &str) {
        let url = format!("https://example.com/{id}.xml");
        db::feeds::insert_feed(
            connection,
            profile_id,
            id,
            &db::feeds::NewFeed {
                title: id,
                site_url: None,
                feed_url: &url,
                feed_url_normalized: &url,
                feed_type: "rss",
                description: None,
                language: None,
                etag: None,
                last_modified: None,
                folder_id: None,
            },
        )
        .unwrap();
    }

    fn source() -> NewSource<'static> {
        NewSource {
            kind: IntegrationKind::Reddit,
            source_key: "rustlang",
            display_name: "RustLang",
            source_url: Some("https://www.reddit.com/r/rustlang/"),
            feed_url: "https://www.reddit.com/r/rustlang/.rss",
            feed_url_normalized: "https://www.reddit.com/r/rustlang/.rss",
            category: Some("Programming"),
            trust_level: "low",
            subscribed_feed_id: None,
        }
    }

    #[test]
    fn sources_are_profile_scoped_and_unique() {
        let (connection, first, second) = setup();
        let created = upsert_source(&connection, &first, None, &source()).unwrap();
        assert_eq!(list_sources(&connection, &first).unwrap().len(), 1);
        assert!(list_sources(&connection, &second).unwrap().is_empty());
        assert!(get_source(&connection, &second, &created.id).is_err());

        let duplicate = upsert_source(&connection, &first, None, &source());
        assert!(duplicate.is_ok(), "same key should reuse the source");

        let mut conflict = source();
        conflict.source_key = "other";
        conflict.feed_url_normalized = "https://www.reddit.com/r/other/.rss";
        conflict.feed_url = conflict.feed_url_normalized;
        let created_other = upsert_source(&connection, &first, None, &conflict).unwrap();
        let conflicting = NewSource {
            source_key: "different",
            display_name: "Different",
            ..source()
        };
        assert!(matches!(
            upsert_source(&connection, &first, Some(&created_other.id), &conflicting),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn source_state_updates_and_deletion_are_profile_scoped() {
        let (connection, first, second) = setup();
        let created = upsert_source(&connection, &first, None, &source()).unwrap();
        create_feed(&connection, &first, "feed-1");
        create_feed(&connection, &second, "feed-2");
        assert!(set_subscription(&connection, &first, &created.id, "feed-1").is_ok());
        assert!(set_subscription(&connection, &second, &created.id, "feed-2").is_err());
        let errored = set_error(&connection, &first, &created.id, "HTTP 500").unwrap();
        assert_eq!(errored.status, "error");
        assert_eq!(errored.last_error.as_deref(), Some("HTTP 500"));
        let active = clear_error(&connection, &first, &created.id).unwrap();
        assert_eq!(active.status, "active");
        assert!(active.last_error.is_none());
        assert!(delete_source(&connection, &second, &created.id).is_err());
        assert!(delete_source(&connection, &first, &created.id).is_ok());
        assert!(list_sources(&connection, &first).unwrap().is_empty());
    }
}
