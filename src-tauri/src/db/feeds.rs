use crate::error::{AppError, AppResult};
use crate::feeds::models::{
    health_for_failures, FeedDto, FeedFolderDto, FeedItemDto, RefreshLogDto,
};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};

pub const MAX_LISTED_FEEDS: usize = 500;

const FEED_COLUMNS: &str = "f.id, f.folder_id, f.title, f.site_url, f.feed_url, f.feed_type,
    f.description, f.language, f.status, f.last_refreshed_at, f.last_success_at,
    f.next_refresh_at, f.refresh_interval_minutes, f.consecutive_failures, f.last_error,
    (SELECT COUNT(*) FROM feed_items i WHERE i.feed_id = f.id) AS article_count,
    f.created_at, f.updated_at, f.category, f.trust_level";

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn map_feed(row: &rusqlite::Row<'_>) -> rusqlite::Result<FeedDto> {
    let failures: i64 = row.get(13)?;
    Ok(FeedDto {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        site_url: row.get(3)?,
        feed_url: row.get(4)?,
        feed_type: row.get(5)?,
        description: row.get(6)?,
        language: row.get(7)?,
        status: row.get(8)?,
        last_refreshed_at: row.get(9)?,
        last_success_at: row.get(10)?,
        next_refresh_at: row.get(11)?,
        refresh_interval_minutes: row.get(12)?,
        consecutive_failures: failures,
        last_error: row.get(14)?,
        article_count: row.get(15)?,
        health: health_for_failures(failures).to_string(),
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        category: row.get(18)?,
        trust_level: row.get(19)?,
    })
}

pub struct NewFeed<'a> {
    pub title: &'a str,
    pub site_url: Option<&'a str>,
    pub feed_url: &'a str,
    pub feed_url_normalized: &'a str,
    pub feed_type: &'a str,
    pub description: Option<&'a str>,
    pub language: Option<&'a str>,
    pub etag: Option<&'a str>,
    pub last_modified: Option<&'a str>,
    pub folder_id: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct NewItem<'a> {
    pub guid: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
    pub author: Option<&'a str>,
    pub summary: Option<&'a str>,
    pub content_html: Option<&'a str>,
    pub published_at: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct FeedFetchData {
    pub title: String,
    pub feed_url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub refresh_interval_minutes: i64,
}

#[derive(Debug, Clone)]
pub struct FeedRef {
    pub profile_id: String,
    pub feed_id: String,
}

fn validate_folder(
    connection: &Connection,
    profile_id: &str,
    folder_id: Option<&str>,
) -> AppResult<()> {
    let Some(folder_id) = folder_id else {
        return Ok(());
    };
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM feed_folders WHERE id = ?1 AND profile_id = ?2)",
        params![folder_id, profile_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::Validation(
            "That folder is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn list_folders(connection: &Connection, profile_id: &str) -> AppResult<Vec<FeedFolderDto>> {
    let mut statement = connection.prepare(
        "SELECT f.id, f.name, f.created_at, f.updated_at,
            (SELECT COUNT(*) FROM feeds fd WHERE fd.folder_id = f.id) AS feed_count
         FROM feed_folders f
         WHERE f.profile_id = ?1
         ORDER BY f.name COLLATE NOCASE ASC",
    )?;
    let folders = statement
        .query_map([profile_id], |row| {
            Ok(FeedFolderDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                feed_count: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(folders)
}

pub fn create_folder(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    name: &str,
    normalized: &str,
) -> AppResult<FeedFolderDto> {
    let timestamp = now();
    connection.execute(
        "INSERT INTO feed_folders (id, profile_id, name, name_normalized, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, profile_id, name, normalized, timestamp, timestamp],
    )?;
    let mut statement = connection.prepare(
        "SELECT f.id, f.name, f.created_at, f.updated_at,
            (SELECT COUNT(*) FROM feeds fd WHERE fd.folder_id = f.id) AS feed_count
         FROM feed_folders f WHERE f.id = ?1 AND f.profile_id = ?2",
    )?;
    statement
        .query_row([id, profile_id], |row| {
            Ok(FeedFolderDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                feed_count: row.get(4)?,
            })
        })
        .map_err(AppError::from)
}

pub fn delete_folder(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM feed_folders WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Folder not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn list_feeds(connection: &Connection, profile_id: &str) -> AppResult<Vec<FeedDto>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {FEED_COLUMNS} FROM feeds f WHERE f.profile_id = ?1
         ORDER BY f.title COLLATE NOCASE ASC LIMIT ?2"
    ))?;
    let feeds = statement
        .query_map(params![profile_id, MAX_LISTED_FEEDS as i64], map_feed)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(feeds)
}

pub fn get_feed(connection: &Connection, profile_id: &str, id: &str) -> AppResult<FeedDto> {
    let mut statement = connection.prepare(&format!(
        "SELECT {FEED_COLUMNS} FROM feeds f WHERE f.profile_id = ?1 AND f.id = ?2"
    ))?;
    statement
        .query_row([profile_id, id], map_feed)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Feed not found in this profile.".to_string()))
}

pub fn find_feed_by_normalized_url(
    connection: &Connection,
    profile_id: &str,
    normalized_url: &str,
) -> AppResult<Option<FeedDto>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {FEED_COLUMNS} FROM feeds f
         WHERE f.profile_id = ?1 AND f.feed_url_normalized = ?2"
    ))?;
    statement
        .query_row([profile_id, normalized_url], map_feed)
        .optional()
        .map_err(AppError::from)
}

pub fn insert_feed(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    feed: &NewFeed<'_>,
) -> AppResult<FeedDto> {
    validate_folder(connection, profile_id, feed.folder_id)?;
    let timestamp = now();
    connection.execute(
        "INSERT INTO feeds (
            id, profile_id, folder_id, title, site_url, feed_url, feed_url_normalized, feed_type,
            description, language, etag, last_modified, next_refresh_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            id,
            profile_id,
            feed.folder_id,
            feed.title,
            feed.site_url,
            feed.feed_url,
            feed.feed_url_normalized,
            feed.feed_type,
            feed.description,
            feed.language,
            feed.etag,
            feed.last_modified,
            timestamp,
            timestamp,
            timestamp,
        ],
    )?;
    get_feed(connection, profile_id, id)
}

pub fn insert_items(
    connection: &Connection,
    profile_id: &str,
    feed_id: &str,
    items: &[NewItem<'_>],
) -> AppResult<usize> {
    let mut inserted = 0usize;
    for item in items {
        let item_id = uuid::Uuid::new_v4().to_string();
        let discovered_at = now();
        let changed = connection.execute(
            "INSERT OR IGNORE INTO feed_items (
                id, profile_id, feed_id, guid, title, url, author, summary, published_at, discovered_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item_id,
                profile_id,
                feed_id,
                item.guid,
                item.title,
                item.url,
                item.author,
                item.summary,
                item.published_at,
                discovered_at,
            ],
        )?;
        if changed == 0 {
            continue;
        }
        crate::articles::store::initialize_item_content(
            connection,
            profile_id,
            &item_id,
            item.summary,
            item.content_html,
            item.url,
        )?;
        inserted += 1;
    }
    Ok(inserted)
}

pub fn subscribe(
    connection: &mut Connection,
    profile_id: &str,
    feed_id: &str,
    feed: &NewFeed<'_>,
    items: &[NewItem<'_>],
) -> AppResult<(FeedDto, usize)> {
    let transaction = connection.transaction()?;
    let stored = insert_feed(&transaction, profile_id, feed_id, feed)?;
    let inserted = insert_items(&transaction, profile_id, feed_id, items)?;
    transaction.commit()?;
    Ok((stored, inserted))
}

pub fn set_organization(
    connection: &Connection,
    profile_id: &str,
    feed_id: &str,
    category: Option<&str>,
    trust_level: &str,
) -> AppResult<FeedDto> {
    let changed = connection.execute(
        "UPDATE feeds SET category = ?1, trust_level = ?2, updated_at = ?3
         WHERE id = ?4 AND profile_id = ?5",
        params![category, trust_level, now(), feed_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That feed is not available in this profile.".to_string(),
        ));
    }
    list_feeds(connection, profile_id).and_then(|feeds| {
        feeds
            .into_iter()
            .find(|feed| feed.id == feed_id)
            .ok_or_else(|| {
                AppError::NotFound("That feed is not available in this profile.".to_string())
            })
    })
}

pub fn update_feed(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    title: &str,
    folder_id: Option<&str>,
    status: &str,
    refresh_interval_minutes: i64,
    category: Option<&str>,
    trust_level: &str,
) -> AppResult<FeedDto> {
    validate_folder(connection, profile_id, folder_id)?;
    let next_refresh_at = now();
    let changed = connection.execute(
        "UPDATE feeds SET title = ?1, folder_id = ?2, status = ?3,
            refresh_interval_minutes = ?4, next_refresh_at = ?5, updated_at = ?6,
            category = ?7, trust_level = ?8
         WHERE id = ?9 AND profile_id = ?10",
        params![
            title,
            folder_id,
            status,
            refresh_interval_minutes,
            next_refresh_at,
            next_refresh_at,
            category,
            trust_level,
            id,
            profile_id,
        ],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Feed not found in this profile.".to_string(),
        ));
    }
    get_feed(connection, profile_id, id)
}

pub fn delete_feed(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM feeds WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Feed not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn get_feed_fetch_data(
    connection: &Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<FeedFetchData> {
    connection
        .query_row(
            "SELECT title, feed_url, etag, last_modified, refresh_interval_minutes
             FROM feeds WHERE id = ?1 AND profile_id = ?2",
            params![id, profile_id],
            |row| {
                Ok(FeedFetchData {
                    title: row.get(0)?,
                    feed_url: row.get(1)?,
                    etag: row.get(2)?,
                    last_modified: row.get(3)?,
                    refresh_interval_minutes: row.get(4)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Feed not found in this profile.".to_string()))
}

fn next_refresh(interval_minutes: i64, failures: i64) -> String {
    let multiplier = 2_i64.pow(failures.clamp(0, 4) as u32);
    (Utc::now() + Duration::minutes(interval_minutes * multiplier)).to_rfc3339()
}

fn insert_refresh_log(
    connection: &Connection,
    profile_id: &str,
    feed_id: &str,
    started_at: &str,
    status: &str,
    http_status: Option<i64>,
    item_count: i64,
    new_item_count: i64,
    duration_ms: i64,
    message: Option<&str>,
) -> AppResult<()> {
    connection.execute(
        "INSERT INTO feed_refresh_logs (
            id, profile_id, feed_id, started_at, finished_at, status, http_status,
            item_count, new_item_count, duration_ms, message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            uuid::Uuid::new_v4().to_string(),
            profile_id,
            feed_id,
            started_at,
            now(),
            status,
            http_status,
            item_count,
            new_item_count,
            duration_ms,
            message,
        ],
    )?;
    Ok(())
}

pub struct RefreshSuccess<'a> {
    pub feed_id: &'a str,
    pub etag: Option<&'a str>,
    pub last_modified: Option<&'a str>,
    pub interval_minutes: i64,
    pub items: &'a [NewItem<'a>],
    pub item_count: i64,
    pub http_status: Option<i64>,
    pub duration_ms: i64,
    pub started_at: &'a str,
}

pub fn record_success(
    connection: &mut Connection,
    profile_id: &str,
    result: RefreshSuccess<'_>,
) -> AppResult<FeedDto> {
    let transaction = connection.transaction()?;
    let new_items = insert_items(&transaction, profile_id, result.feed_id, result.items)?;
    let refreshed_at = now();
    let next = next_refresh(result.interval_minutes, 0);
    transaction.execute(
        "UPDATE feeds SET etag = ?1, last_modified = ?2, last_refreshed_at = ?3,
            last_success_at = ?4, next_refresh_at = ?5, consecutive_failures = 0,
            last_error = NULL, updated_at = ?6
         WHERE id = ?7 AND profile_id = ?8",
        params![
            result.etag,
            result.last_modified,
            refreshed_at,
            refreshed_at,
            next,
            refreshed_at,
            result.feed_id,
            profile_id,
        ],
    )?;
    insert_refresh_log(
        &transaction,
        profile_id,
        result.feed_id,
        result.started_at,
        "success",
        result.http_status,
        result.item_count,
        new_items as i64,
        result.duration_ms,
        None,
    )?;
    transaction.commit()?;
    get_feed(connection, profile_id, result.feed_id)
}

pub struct RefreshNotModified<'a> {
    pub feed_id: &'a str,
    pub interval_minutes: i64,
    pub http_status: Option<i64>,
    pub duration_ms: i64,
    pub started_at: &'a str,
}

pub fn record_not_modified(
    connection: &mut Connection,
    profile_id: &str,
    result: RefreshNotModified<'_>,
) -> AppResult<FeedDto> {
    let transaction = connection.transaction()?;
    let refreshed_at = now();
    let next = next_refresh(result.interval_minutes, 0);
    transaction.execute(
        "UPDATE feeds SET last_refreshed_at = ?1, next_refresh_at = ?2,
            consecutive_failures = 0, last_error = NULL, updated_at = ?3
         WHERE id = ?4 AND profile_id = ?5",
        params![refreshed_at, next, refreshed_at, result.feed_id, profile_id],
    )?;
    insert_refresh_log(
        &transaction,
        profile_id,
        result.feed_id,
        result.started_at,
        "not_modified",
        result.http_status,
        0,
        0,
        result.duration_ms,
        None,
    )?;
    transaction.commit()?;
    get_feed(connection, profile_id, result.feed_id)
}

pub struct RefreshFailure<'a> {
    pub feed_id: &'a str,
    pub interval_minutes: i64,
    pub failures: i64,
    pub http_status: Option<i64>,
    pub duration_ms: i64,
    pub started_at: &'a str,
    pub message: &'a str,
}

pub fn record_failure(
    connection: &mut Connection,
    profile_id: &str,
    result: RefreshFailure<'_>,
) -> AppResult<FeedDto> {
    let transaction = connection.transaction()?;
    let refreshed_at = now();
    let failures = result.failures + 1;
    let next = next_refresh(result.interval_minutes, failures);
    transaction.execute(
        "UPDATE feeds SET last_refreshed_at = ?1, next_refresh_at = ?2,
            consecutive_failures = ?3, last_error = ?4, updated_at = ?5
         WHERE id = ?6 AND profile_id = ?7",
        params![
            refreshed_at,
            next,
            failures,
            result.message,
            refreshed_at,
            result.feed_id,
            profile_id,
        ],
    )?;
    insert_refresh_log(
        &transaction,
        profile_id,
        result.feed_id,
        result.started_at,
        "error",
        result.http_status,
        0,
        0,
        result.duration_ms,
        Some(result.message),
    )?;
    transaction.commit()?;
    get_feed(connection, profile_id, result.feed_id)
}

pub fn list_refresh_logs(
    connection: &Connection,
    profile_id: &str,
    feed_id: &str,
    limit: usize,
) -> AppResult<Vec<RefreshLogDto>> {
    let mut statement = connection.prepare(
        "SELECT id, feed_id, started_at, finished_at, status, http_status,
            item_count, new_item_count, duration_ms, message
         FROM feed_refresh_logs
         WHERE profile_id = ?1 AND feed_id = ?2
         ORDER BY started_at DESC LIMIT ?3",
    )?;
    let logs = statement
        .query_map(params![profile_id, feed_id, limit as i64], |row| {
            Ok(RefreshLogDto {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                status: row.get(4)?,
                http_status: row.get(5)?,
                item_count: row.get(6)?,
                new_item_count: row.get(7)?,
                duration_ms: row.get(8)?,
                message: row.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(logs)
}

pub fn due_feeds(connection: &Connection, limit: usize) -> AppResult<Vec<FeedRef>> {
    let mut statement = connection.prepare(
        "SELECT profile_id, id FROM feeds
         WHERE status = 'active' AND next_refresh_at <= ?1
         ORDER BY next_refresh_at ASC LIMIT ?2",
    )?;
    let feeds = statement
        .query_map(params![now(), limit as i64], |row| {
            Ok(FeedRef {
                profile_id: row.get(0)?,
                feed_id: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(feeds)
}

pub fn due_feeds_for_profile(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<FeedRef>> {
    let mut statement = connection.prepare(
        "SELECT profile_id, id FROM feeds
         WHERE profile_id = ?1 AND status = 'active' AND next_refresh_at <= ?2
         ORDER BY next_refresh_at ASC LIMIT ?3",
    )?;
    let feeds = statement
        .query_map(params![profile_id, now(), limit as i64], |row| {
            Ok(FeedRef {
                profile_id: row.get(0)?,
                feed_id: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(feeds)
}

pub fn recent_items(
    connection: &Connection,
    profile_id: &str,
    feed_id: &str,
    limit: usize,
) -> AppResult<Vec<FeedItemDto>> {
    let mut statement = connection.prepare(
        "SELECT id, feed_id, guid, title, url, author, summary, published_at, discovered_at
         FROM feed_items WHERE profile_id = ?1 AND feed_id = ?2
         ORDER BY COALESCE(published_at, discovered_at) DESC LIMIT ?3",
    )?;
    let items = statement
        .query_map(params![profile_id, feed_id, limit as i64], |row| {
            Ok(FeedItemDto {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                guid: row.get(2)?,
                title: row.get(3)?,
                url: row.get(4)?,
                author: row.get(5)?,
                summary: row.get(6)?,
                published_at: row.get(7)?,
                discovered_at: row.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, profiles, settings};

    fn setup() -> (Connection, String, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::db::configure(&connection).unwrap();
        migrations::run(&mut connection).unwrap();
        profiles::ensure_default_profile(&mut connection).unwrap();
        let first = settings::get_active_profile_id(&connection).unwrap();
        let second = profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    fn new_feed<'a>(url: &'a str) -> NewFeed<'a> {
        NewFeed {
            title: "Example",
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

    #[test]
    fn folders_and_feeds_are_profile_scoped() {
        let (mut connection, first, second) = setup();
        create_folder(&connection, &first, "folder-1", "News", "news").unwrap();
        assert_eq!(list_folders(&connection, &first).unwrap().len(), 1);
        assert!(list_folders(&connection, &second).unwrap().is_empty());

        subscribe(
            &mut connection,
            &first,
            "feed-1",
            &new_feed("https://example.com/feed"),
            &[],
        )
        .unwrap();
        assert_eq!(list_feeds(&connection, &first).unwrap().len(), 1);
        assert!(list_feeds(&connection, &second).unwrap().is_empty());
        assert!(get_feed(&connection, &second, "feed-1").is_err());
    }

    #[test]
    fn duplicate_feed_url_is_rejected_within_a_profile() {
        let (mut connection, first, _) = setup();
        subscribe(
            &mut connection,
            &first,
            "feed-1",
            &new_feed("https://example.com/feed"),
            &[],
        )
        .unwrap();
        assert!(subscribe(
            &mut connection,
            &first,
            "feed-2",
            &new_feed("https://example.com/feed"),
            &[]
        )
        .is_err());
    }

    #[test]
    fn items_are_deduplicated_by_guid() {
        let (mut connection, first, _) = setup();
        subscribe(
            &mut connection,
            &first,
            "feed-1",
            &new_feed("https://example.com/feed"),
            &[],
        )
        .unwrap();
        let item = NewItem {
            guid: "item-1",
            title: "First",
            url: None,
            author: None,
            summary: None,
            content_html: None,
            published_at: None,
        };
        assert_eq!(
            insert_items(&connection, &first, "feed-1", &[item.clone()]).unwrap(),
            1
        );
        assert_eq!(
            insert_items(&connection, &first, "feed-1", &[item]).unwrap(),
            0
        );
        assert_eq!(
            recent_items(&connection, &first, "feed-1", 10)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn failures_apply_backoff_and_write_refresh_logs() {
        let (mut connection, first, _) = setup();
        subscribe(
            &mut connection,
            &first,
            "feed-1",
            &new_feed("https://example.com/feed"),
            &[],
        )
        .unwrap();
        let before = get_feed(&connection, &first, "feed-1")
            .unwrap()
            .next_refresh_at;
        record_failure(
            &mut connection,
            &first,
            RefreshFailure {
                feed_id: "feed-1",
                interval_minutes: 60,
                failures: 0,
                http_status: Some(500),
                duration_ms: 12,
                started_at: &now(),
                message: "The feed server returned HTTP 500.",
            },
        )
        .unwrap();
        let after = get_feed(&connection, &first, "feed-1").unwrap();
        assert_eq!(after.consecutive_failures, 1);
        assert_eq!(after.health, "degraded");
        assert!(after.next_refresh_at > before);
        let logs = list_refresh_logs(&connection, &first, "feed-1", 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "error");
    }

    #[test]
    fn deleting_profile_cascades_feed_data() {
        let (mut connection, first, _) = setup();
        create_folder(&connection, &first, "folder-1", "News", "news").unwrap();
        let feed = new_feed("https://example.com/feed");
        subscribe(&mut connection, &first, "feed-1", &feed, &[]).unwrap();
        profiles::delete_profile(&mut connection, &first).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM feed_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM feeds", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM feed_folders", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
