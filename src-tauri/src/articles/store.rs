use super::content;
use super::extraction::CachedImage;
use super::models::{
    ArticleCacheStatsDto, ArticleDetailDto, ArticleHighlightDto, ArticleImageDto,
    ArticleListItemDto, ArticleNoteDto, ArticleStateDto, ArticleStatePatch, CollectionDto,
    HighlightEntryDto, HistoryEntryDto,
};
use crate::error::{AppError, AppResult};
use crate::validation::normalize_display_name;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

fn now() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) const ARTICLE_COLUMNS: &str = "i.id, i.feed_id, f.title, i.title, i.url, i.author,
    i.summary, i.published_at, i.discovered_at, COALESCE(src.word_count, 0),
    COALESCE(s.is_read, 0), COALESCE(s.is_saved, 0), COALESCE(s.is_favorite, 0),
    COALESCE(s.is_archived, 0), COALESCE(s.is_queued, 0), s.queue_position,
    COALESCE(s.read_count, 0), s.last_read_at";

pub(crate) const ARTICLE_FROM: &str = "feed_items i
  JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
  LEFT JOIN article_sources src ON src.item_id = i.id AND src.profile_id = i.profile_id
  LEFT JOIN article_states s ON s.item_id = i.id AND s.profile_id = i.profile_id";

pub(crate) fn map_article(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArticleListItemDto> {
    Ok(ArticleListItemDto {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        feed_title: row.get(2)?,
        title: row.get(3)?,
        url: row.get(4)?,
        author: row.get(5)?,
        summary: row.get(6)?,
        published_at: row.get(7)?,
        discovered_at: row.get(8)?,
        word_count: row.get(9)?,
        state: ArticleStateDto {
            is_read: row.get::<_, i64>(10)? != 0,
            is_saved: row.get::<_, i64>(11)? != 0,
            is_favorite: row.get::<_, i64>(12)? != 0,
            is_archived: row.get::<_, i64>(13)? != 0,
            is_queued: row.get::<_, i64>(14)? != 0,
            queue_position: row.get(15)?,
            read_count: row.get(16)?,
            last_read_at: row.get(17)?,
        },
    })
}

fn item_exists(connection: &Connection, profile_id: &str, item_id: &str) -> AppResult<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM feed_items WHERE id = ?1 AND profile_id = ?2)",
        params![item_id, profile_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound(
            "Article not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn get_item_url(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
) -> AppResult<Option<String>> {
    connection
        .query_row(
            "SELECT url FROM feed_items WHERE id = ?1 AND profile_id = ?2",
            params![item_id, profile_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)
}

pub fn ensure_state(connection: &Connection, profile_id: &str, item_id: &str) -> AppResult<()> {
    let timestamp = now();
    connection.execute(
        "INSERT OR IGNORE INTO article_states (
            item_id, profile_id, is_read, is_saved, is_favorite, is_archived,
            is_queued, queue_position, read_count, created_at, updated_at
         ) VALUES (?1, ?2, 0, 0, 0, 0, 0, NULL, 0, ?3, ?3)",
        params![item_id, profile_id, timestamp],
    )?;
    Ok(())
}

fn hash_url(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub struct StoredSource<'a> {
    pub raw_html: Option<&'a str>,
    pub content_text: &'a str,
    pub safe_html: &'a str,
    pub word_count: i64,
    pub image_urls: &'a [String],
    pub images: &'a [CachedImage],
    pub source: &'a str,
}

pub fn save_source(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    source: &StoredSource<'_>,
) -> AppResult<()> {
    item_exists(connection, profile_id, item_id)?;
    let image_json = serde_json::to_string(source.image_urls)?;
    let timestamp = now();
    connection.execute(
        "INSERT INTO article_sources (
            item_id, profile_id, raw_html, content_text, safe_html, word_count,
            image_urls, extracted_at, source
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(item_id) DO UPDATE SET
            raw_html = excluded.raw_html, content_text = excluded.content_text,
            safe_html = excluded.safe_html, word_count = excluded.word_count,
            image_urls = excluded.image_urls, extracted_at = excluded.extracted_at,
            source = excluded.source",
        params![
            item_id,
            profile_id,
            source.raw_html,
            source.content_text,
            source.safe_html,
            source.word_count,
            image_json,
            timestamp,
            source.source,
        ],
    )?;
    connection.execute(
        "DELETE FROM article_images WHERE item_id = ?1 AND profile_id = ?2",
        params![item_id, profile_id],
    )?;
    for image in source.images {
        connection.execute(
            "INSERT INTO article_images (
                item_id, profile_id, url_hash, url, mime, data, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item_id,
                profile_id,
                hash_url(&image.url),
                image.url,
                image.mime,
                image.bytes,
                timestamp,
            ],
        )?;
    }
    Ok(())
}

pub fn initialize_item_content(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    summary: Option<&str>,
    content_html: Option<&str>,
    source_url: Option<&str>,
) -> AppResult<()> {
    ensure_state(connection, profile_id, item_id)?;
    let base = source_url.and_then(|value| url::Url::parse(value).ok());
    let mut parsed = match content_html.filter(|value| !value.trim().is_empty()) {
        Some(html) => content::sanitize(html, base.as_ref()),
        None => content::plain_text_source(summary.unwrap_or("")),
    };
    parsed.safe_html = content::rewrite_image_sources(&parsed.safe_html, &HashMap::new());
    parsed.image_urls.clear();
    save_source(
        connection,
        profile_id,
        item_id,
        &StoredSource {
            raw_html: content_html,
            content_text: &parsed.text,
            safe_html: &parsed.safe_html,
            word_count: parsed.word_count,
            image_urls: &parsed.image_urls,
            images: &[],
            source: "feed",
        },
    )
}

pub fn list_articles(
    connection: &Connection,
    profile_id: &str,
    mode: &str,
    collection_id: Option<&str>,
    limit: usize,
) -> AppResult<Vec<ArticleListItemDto>> {
    let (filter, order) = match mode {
        "all" => ("", "COALESCE(i.published_at, i.discovered_at) DESC"),
        "unread" => (
            "AND COALESCE(s.is_read, 0) = 0 AND COALESCE(s.is_archived, 0) = 0",
            "COALESCE(i.published_at, i.discovered_at) DESC",
        ),
        "saved" => (
            "AND COALESCE(s.is_saved, 0) = 1",
            "COALESCE(i.published_at, i.discovered_at) DESC",
        ),
        "favorites" => (
            "AND COALESCE(s.is_favorite, 0) = 1",
            "COALESCE(i.published_at, i.discovered_at) DESC",
        ),
        "archive" => (
            "AND COALESCE(s.is_archived, 0) = 1",
            "COALESCE(i.published_at, i.discovered_at) DESC",
        ),
        "history" => (
            "AND EXISTS (SELECT 1 FROM article_history h WHERE h.item_id = i.id AND h.profile_id = i.profile_id)",
            "(SELECT MAX(h.occurred_at) FROM article_history h WHERE h.item_id = i.id AND h.profile_id = i.profile_id) DESC",
        ),
        "queue" => (
            "AND COALESCE(s.is_queued, 0) = 1",
            "CASE WHEN s.queue_position IS NULL THEN 1 ELSE 0 END, s.queue_position ASC, i.discovered_at ASC",
        ),
        "highlights" => (
            "AND EXISTS (SELECT 1 FROM article_highlights h WHERE h.item_id = i.id AND h.profile_id = i.profile_id)",
            "COALESCE(i.published_at, i.discovered_at) DESC",
        ),
        "collection" => {
            if collection_id.is_none() {
                return Err(AppError::Validation("Choose a collection first.".to_string()));
            }
            (
                "AND EXISTS (SELECT 1 FROM collection_items ci WHERE ci.item_id = i.id AND ci.profile_id = i.profile_id AND ci.collection_id = ?2)",
                "COALESCE(i.published_at, i.discovered_at) DESC",
            )
        }
        _ => return Err(AppError::Validation("Unknown article list mode.".to_string())),
    };

    let (sql, values) = if mode == "collection" {
        (
            format!(
                "SELECT {ARTICLE_COLUMNS} FROM {ARTICLE_FROM}
                 WHERE i.profile_id = ?1 {filter} ORDER BY {order} LIMIT ?3"
            ),
            vec![
                rusqlite::types::Value::Text(profile_id.to_string()),
                rusqlite::types::Value::Text(collection_id.unwrap_or_default().to_string()),
                rusqlite::types::Value::Integer(limit as i64),
            ],
        )
    } else {
        (
            format!(
                "SELECT {ARTICLE_COLUMNS} FROM {ARTICLE_FROM}
                 WHERE i.profile_id = ?1 {filter} ORDER BY {order} LIMIT ?2"
            ),
            vec![
                rusqlite::types::Value::Text(profile_id.to_string()),
                rusqlite::types::Value::Integer(limit as i64),
            ],
        )
    };
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(values.iter()), map_article)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_articles_by_ids(
    connection: &Connection,
    profile_id: &str,
    item_ids: &[String],
    limit: usize,
) -> AppResult<Vec<ArticleListItemDto>> {
    if item_ids.is_empty() || item_ids.len() > 200 {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; item_ids.len()].join(", ");
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS} FROM {ARTICLE_FROM}
         WHERE i.profile_id = ?1 AND i.id IN ({placeholders})
         ORDER BY COALESCE(i.published_at, i.discovered_at) DESC LIMIT ?{}",
        item_ids.len() + 2
    );
    let mut values = vec![rusqlite::types::Value::Text(profile_id.to_string())];
    values.extend(
        item_ids
            .iter()
            .map(|id| rusqlite::types::Value::Text(id.clone())),
    );
    values.push(rusqlite::types::Value::Integer(limit as i64));
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(rusqlite::params_from_iter(values.iter()), map_article)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn map_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArticleStateDto> {
    Ok(ArticleStateDto {
        is_read: row.get::<_, i64>(0)? != 0,
        is_saved: row.get::<_, i64>(1)? != 0,
        is_favorite: row.get::<_, i64>(2)? != 0,
        is_archived: row.get::<_, i64>(3)? != 0,
        is_queued: row.get::<_, i64>(4)? != 0,
        queue_position: row.get(5)?,
        read_count: row.get(6)?,
        last_read_at: row.get(7)?,
    })
}

fn current_state(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
) -> AppResult<ArticleStateDto> {
    ensure_state(connection, profile_id, item_id)?;
    connection
        .query_row(
            "SELECT is_read, is_saved, is_favorite, is_archived, is_queued,
            queue_position, read_count, last_read_at
         FROM article_states WHERE item_id = ?1 AND profile_id = ?2",
            params![item_id, profile_id],
            map_state,
        )
        .map_err(AppError::from)
}

pub fn apply_patch(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    patch: &ArticleStatePatch,
) -> AppResult<ArticleStateDto> {
    item_exists(connection, profile_id, item_id)?;
    let before = current_state(connection, profile_id, item_id)?;
    let timestamp = now();
    let mut queued = patch.is_queued.unwrap_or(before.is_queued);
    let mut queue_position = before.queue_position;
    if patch.is_queued == Some(true) && !before.is_queued {
        queue_position = connection.query_row(
            "SELECT COALESCE(MAX(queue_position), 0) + 1 FROM article_states
             WHERE profile_id = ?1 AND is_queued = 1",
            [profile_id],
            |row| row.get(0),
        )?;
    }
    if patch.is_queued == Some(false) {
        queued = false;
        queue_position = None;
    }
    connection.execute(
        "UPDATE article_states SET
            is_read = ?1, is_saved = ?2, is_favorite = ?3, is_archived = ?4,
            is_queued = ?5, queue_position = ?6, updated_at = ?7
         WHERE item_id = ?8 AND profile_id = ?9",
        params![
            patch.is_read.unwrap_or(before.is_read) as i64,
            patch.is_saved.unwrap_or(before.is_saved) as i64,
            patch.is_favorite.unwrap_or(before.is_favorite) as i64,
            patch.is_archived.unwrap_or(before.is_archived) as i64,
            queued as i64,
            queue_position,
            timestamp,
            item_id,
            profile_id,
        ],
    )?;

    if let Some(is_read) = patch.is_read {
        if is_read != before.is_read {
            let event = if is_read { "read" } else { "unread" };
            connection.execute(
                "INSERT INTO article_history (id, profile_id, item_id, event_type, occurred_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    profile_id,
                    item_id,
                    event,
                    timestamp
                ],
            )?;
            connection.execute(
                "UPDATE article_states SET read_count = read_count + 1, last_read_at = ?1
                 WHERE item_id = ?2 AND profile_id = ?3",
                params![timestamp, item_id, profile_id],
            )?;
        }
    }
    current_state(connection, profile_id, item_id)
}

pub fn update_state(
    connection: &mut Connection,
    profile_id: &str,
    item_id: &str,
    patch: &ArticleStatePatch,
) -> AppResult<ArticleStateDto> {
    if patch.is_empty() {
        return Err(AppError::Validation(
            "No article state was provided.".to_string(),
        ));
    }
    let transaction = connection.transaction()?;
    let state = apply_patch(&transaction, profile_id, item_id, patch)?;
    transaction.commit()?;
    Ok(state)
}

pub fn bulk_update_state(
    connection: &mut Connection,
    profile_id: &str,
    item_ids: &[String],
    patch: &ArticleStatePatch,
) -> AppResult<usize> {
    if patch.is_empty() {
        return Err(AppError::Validation(
            "No article state was provided.".to_string(),
        ));
    }
    if item_ids.is_empty() || item_ids.len() > 200 {
        return Err(AppError::Validation(
            "Select between 1 and 200 articles.".to_string(),
        ));
    }
    let transaction = connection.transaction()?;
    let mut changed = 0usize;
    for item_id in item_ids {
        if item_exists(&transaction, profile_id, item_id).is_ok() {
            apply_patch(&transaction, profile_id, item_id, patch)?;
            changed += 1;
        }
    }
    transaction.commit()?;
    Ok(changed)
}

pub fn record_open(
    connection: &mut Connection,
    profile_id: &str,
    item_id: &str,
    mark_read: bool,
) -> AppResult<()> {
    item_exists(connection, profile_id, item_id)?;
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO article_history (id, profile_id, item_id, event_type, occurred_at)
         VALUES (?1, ?2, ?3, 'opened', ?4)",
        params![uuid::Uuid::new_v4().to_string(), profile_id, item_id, now()],
    )?;
    if mark_read {
        apply_patch(
            &transaction,
            profile_id,
            item_id,
            &ArticleStatePatch {
                is_read: Some(true),
                ..ArticleStatePatch::default()
            },
        )?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn list_history(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<HistoryEntryDto>> {
    let mut statement = connection.prepare(
        "SELECT h.id, h.item_id, i.title, f.title, h.event_type, h.occurred_at
         FROM article_history h
         JOIN feed_items i ON i.id = h.item_id AND i.profile_id = h.profile_id
         JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
         WHERE h.profile_id = ?1
         ORDER BY h.occurred_at DESC LIMIT ?2",
    )?;
    let entries = statement
        .query_map(params![profile_id, limit as i64], |row| {
            Ok(HistoryEntryDto {
                id: row.get(0)?,
                item_id: row.get(1)?,
                title: row.get(2)?,
                feed_title: row.get(3)?,
                event_type: row.get(4)?,
                occurred_at: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entries)
}

pub fn get_detail(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
) -> AppResult<ArticleDetailDto> {
    let mut statement = connection.prepare(&format!(
        "SELECT {ARTICLE_COLUMNS} FROM {ARTICLE_FROM} WHERE i.profile_id = ?1 AND i.id = ?2"
    ))?;
    let item = statement
        .query_row(params![profile_id, item_id], map_article)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Article not found in this profile.".to_string()))?;

    let (content_text, safe_html, source): (String, String, String) = connection
        .query_row(
            "SELECT content_text, safe_html, source FROM article_sources
             WHERE profile_id = ?1 AND item_id = ?2",
            params![profile_id, item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .unwrap_or_else(|| {
            let fallback = content::plain_text_source(item.summary.as_deref().unwrap_or(""));
            (fallback.text, fallback.safe_html, "feed".to_string())
        });

    let mut image_statement = connection.prepare(
        "SELECT url, mime, data FROM article_images
         WHERE profile_id = ?1 AND item_id = ?2 ORDER BY created_at ASC",
    )?;
    let images = image_statement
        .query_map(params![profile_id, item_id], |row| {
            let url: String = row.get(0)?;
            let mime: String = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;
            use base64::Engine;
            let data_url = format!(
                "data:{mime};base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            );
            Ok(ArticleImageDto {
                url,
                mime,
                data_url,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut note_statement = connection.prepare(
        "SELECT id, body, created_at, updated_at FROM article_notes
         WHERE profile_id = ?1 AND item_id = ?2 ORDER BY created_at DESC",
    )?;
    let notes = note_statement
        .query_map(params![profile_id, item_id], |row| {
            Ok(ArticleNoteDto {
                id: row.get(0)?,
                body: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut highlight_statement = connection.prepare(
        "SELECT id, text, start_offset, end_offset, created_at FROM article_highlights
         WHERE profile_id = ?1 AND item_id = ?2 ORDER BY start_offset ASC",
    )?;
    let highlights = highlight_statement
        .query_map(params![profile_id, item_id], |row| {
            Ok(ArticleHighlightDto {
                id: row.get(0)?,
                text: row.get(1)?,
                start_offset: row.get(2)?,
                end_offset: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut collection_statement = connection.prepare(
        "SELECT c.id, c.name, c.description, c.created_at, c.updated_at,
            (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.id AND ci.profile_id = c.profile_id)
         FROM collections c
         WHERE c.profile_id = ?1
           AND EXISTS (SELECT 1 FROM collection_items ci
                       WHERE ci.collection_id = c.id AND ci.item_id = ?2 AND ci.profile_id = c.profile_id)
         ORDER BY c.name COLLATE NOCASE ASC",
    )?;
    let collections = collection_statement
        .query_map(params![profile_id, item_id], map_collection)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(ArticleDetailDto {
        item,
        source,
        content_text,
        safe_html,
        images,
        notes,
        highlights,
        collections,
    })
}

pub fn add_note(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    body: &str,
) -> AppResult<ArticleNoteDto> {
    let body = body.trim();
    if body.is_empty() || body.chars().count() > 20_000 {
        return Err(AppError::Validation(
            "Notes must be between 1 and 20000 characters.".to_string(),
        ));
    }
    item_exists(connection, profile_id, item_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = now();
    connection.execute(
        "INSERT INTO article_notes (id, profile_id, item_id, body, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, profile_id, item_id, body, timestamp],
    )?;
    Ok(ArticleNoteDto {
        id,
        body: body.to_string(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn update_note(
    connection: &Connection,
    profile_id: &str,
    note_id: &str,
    body: &str,
) -> AppResult<ArticleNoteDto> {
    let body = body.trim();
    if body.is_empty() || body.chars().count() > 20_000 {
        return Err(AppError::Validation(
            "Notes must be between 1 and 20000 characters.".to_string(),
        ));
    }
    let timestamp = now();
    let changed = connection.execute(
        "UPDATE article_notes SET body = ?1, updated_at = ?2
         WHERE id = ?3 AND profile_id = ?4",
        params![body, timestamp, note_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Note not found in this profile.".to_string(),
        ));
    }
    let created_at: String = connection.query_row(
        "SELECT created_at FROM article_notes WHERE id = ?1 AND profile_id = ?2",
        params![note_id, profile_id],
        |row| row.get(0),
    )?;
    Ok(ArticleNoteDto {
        id: note_id.to_string(),
        body: body.to_string(),
        created_at,
        updated_at: timestamp,
    })
}

pub fn delete_note(connection: &Connection, profile_id: &str, note_id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM article_notes WHERE id = ?1 AND profile_id = ?2",
        params![note_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Note not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn add_highlight(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    text: &str,
    start_offset: i64,
    end_offset: i64,
) -> AppResult<ArticleHighlightDto> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > 2000 {
        return Err(AppError::Validation(
            "Highlights must be between 1 and 2000 characters.".to_string(),
        ));
    }
    if start_offset < 0 || end_offset < start_offset {
        return Err(AppError::Validation(
            "Highlight offsets are invalid.".to_string(),
        ));
    }
    item_exists(connection, profile_id, item_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = now();
    connection.execute(
        "INSERT INTO article_highlights (
            id, profile_id, item_id, text, start_offset, end_offset, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            profile_id,
            item_id,
            text,
            start_offset,
            end_offset,
            timestamp
        ],
    )?;
    Ok(ArticleHighlightDto {
        id,
        text: text.to_string(),
        start_offset,
        end_offset,
        created_at: timestamp,
    })
}

pub fn delete_highlight(
    connection: &Connection,
    profile_id: &str,
    highlight_id: &str,
) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM article_highlights WHERE id = ?1 AND profile_id = ?2",
        params![highlight_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Highlight not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn list_highlights(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<HighlightEntryDto>> {
    let mut statement = connection.prepare(
        "SELECT h.id, h.item_id, i.title, f.title, h.text, h.created_at
         FROM article_highlights h
         JOIN feed_items i ON i.id = h.item_id AND i.profile_id = h.profile_id
         JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
         WHERE h.profile_id = ?1
         ORDER BY h.created_at DESC LIMIT ?2",
    )?;
    let highlights = statement
        .query_map(params![profile_id, limit as i64], |row| {
            Ok(HighlightEntryDto {
                id: row.get(0)?,
                item_id: row.get(1)?,
                article_title: row.get(2)?,
                feed_title: row.get(3)?,
                text: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(highlights)
}

fn map_collection(row: &rusqlite::Row<'_>) -> rusqlite::Result<CollectionDto> {
    Ok(CollectionDto {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        article_count: row.get(5)?,
    })
}

const COLLECTION_SELECT: &str = "SELECT c.id, c.name, c.description, c.created_at, c.updated_at,
    (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.id AND ci.profile_id = c.profile_id)
  FROM collections c";

pub fn list_collections(
    connection: &Connection,
    profile_id: &str,
) -> AppResult<Vec<CollectionDto>> {
    let mut statement = connection.prepare(&format!(
        "{COLLECTION_SELECT} WHERE c.profile_id = ?1 ORDER BY c.name COLLATE NOCASE ASC"
    ))?;
    let collections = statement
        .query_map([profile_id], map_collection)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(collections)
}

pub fn create_collection(
    connection: &Connection,
    profile_id: &str,
    name: &str,
    description: Option<&str>,
) -> AppResult<CollectionDto> {
    let (name, normalized) = normalize_display_name(name, "Collection name")?;
    let description = description
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() > 500 {
                Err(AppError::Validation(
                    "Collection description is too long.".to_string(),
                ))
            } else {
                Ok(value.to_string())
            }
        })
        .transpose()?;
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = now();
    connection.execute(
        "INSERT INTO collections (id, profile_id, name, name_normalized, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, profile_id, name, normalized, description, timestamp],
    )?;
    let mut statement = connection.prepare(&format!(
        "{COLLECTION_SELECT} WHERE c.profile_id = ?1 AND c.id = ?2"
    ))?;
    statement
        .query_row(params![profile_id, id], map_collection)
        .map_err(AppError::from)
}

pub fn delete_collection(
    connection: &Connection,
    profile_id: &str,
    collection_id: &str,
) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM collections WHERE id = ?1 AND profile_id = ?2",
        params![collection_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "Collection not found in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn add_to_collection(
    connection: &Connection,
    profile_id: &str,
    collection_id: &str,
    item_id: &str,
) -> AppResult<()> {
    let collection_exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM collections WHERE id = ?1 AND profile_id = ?2)",
        params![collection_id, profile_id],
        |row| row.get(0),
    )?;
    if !collection_exists {
        return Err(AppError::NotFound(
            "Collection not found in this profile.".to_string(),
        ));
    }
    item_exists(connection, profile_id, item_id)?;
    connection.execute(
        "INSERT OR IGNORE INTO collection_items (collection_id, item_id, profile_id, added_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![collection_id, item_id, profile_id, now()],
    )?;
    Ok(())
}

pub fn remove_from_collection(
    connection: &Connection,
    profile_id: &str,
    collection_id: &str,
    item_id: &str,
) -> AppResult<()> {
    connection.execute(
        "DELETE FROM collection_items
         WHERE collection_id = ?1 AND item_id = ?2 AND profile_id = ?3",
        params![collection_id, item_id, profile_id],
    )?;
    Ok(())
}

pub fn cache_stats(connection: &Connection, profile_id: &str) -> AppResult<ArticleCacheStatsDto> {
    connection.query_row(
        "SELECT
            (SELECT COUNT(*) FROM article_sources WHERE profile_id = ?1),
            (SELECT COALESCE(SUM(LENGTH(COALESCE(raw_html, '')) + LENGTH(content_text) + LENGTH(safe_html)), 0) FROM article_sources WHERE profile_id = ?1),
            (SELECT COUNT(*) FROM article_images WHERE profile_id = ?1),
            (SELECT COALESCE(SUM(LENGTH(data)), 0) FROM article_images WHERE profile_id = ?1)",
        [profile_id],
        |row| {
            Ok(ArticleCacheStatsDto {
                articles: row.get(0)?,
                article_bytes: row.get(1)?,
                images: row.get(2)?,
                image_bytes: row.get(3)?,
            })
        },
    ).map_err(AppError::from)
}

pub fn clear_cache(
    connection: &mut Connection,
    profile_id: &str,
    item_id: Option<&str>,
) -> AppResult<usize> {
    let fallback_summary: Option<String> = match item_id {
        Some(id) => connection
            .query_row(
                "SELECT summary FROM feed_items WHERE id = ?1 AND profile_id = ?2",
                params![id, profile_id],
                |row| row.get(0),
            )
            .optional()?,
        None => None,
    };
    let transaction = connection.transaction()?;
    let changed = match item_id {
        Some(id) => {
            let content = content::plain_text_source(fallback_summary.as_deref().unwrap_or(""));
            transaction.execute(
                "DELETE FROM article_images WHERE item_id = ?1 AND profile_id = ?2",
                params![id, profile_id],
            )?;
            transaction.execute(
                "UPDATE article_sources SET raw_html = NULL, content_text = ?1,
                    safe_html = ?2, word_count = ?3, image_urls = '[]',
                    extracted_at = ?4, source = 'feed'
                 WHERE item_id = ?5 AND profile_id = ?6",
                params![
                    content.text,
                    content.safe_html,
                    content.word_count,
                    now(),
                    id,
                    profile_id
                ],
            )?
        }
        None => {
            transaction.execute(
                "DELETE FROM article_images WHERE profile_id = ?1",
                [profile_id],
            )?;
            transaction.execute(
                "DELETE FROM article_sources WHERE profile_id = ?1",
                [profile_id],
            )?
        }
    };
    transaction.commit()?;
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{feeds as feed_db, migrations, profiles, settings};
    use crate::error::AppError;

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

    fn setup_feed(connection: &mut Connection, profile_id: &str) -> String {
        let feed = feed_db::subscribe(
            connection,
            profile_id,
            "feed-1",
            &feed_db::NewFeed {
                title: "Example",
                site_url: None,
                feed_url: "https://example.com/feed",
                feed_url_normalized: "https://example.com/feed",
                feed_type: "rss",
                description: None,
                language: None,
                etag: None,
                last_modified: None,
                folder_id: None,
            },
            &[],
        )
        .unwrap();
        feed.0.id
    }

    fn insert_item(connection: &Connection, profile_id: &str, feed_id: &str) -> String {
        feed_db::insert_items(
            connection,
            profile_id,
            feed_id,
            &[feed_db::NewItem {
                guid: "item-1",
                title: "First article",
                url: Some("https://example.com/post"),
                author: None,
                summary: Some("Plain summary"),
                content_html: Some("<p>Hello <b>reader</b><script>alert(1)</script></p>"),
                published_at: None,
            }],
        )
        .unwrap();
        feed_db::recent_items(connection, profile_id, feed_id, 10).unwrap()[0]
            .id
            .clone()
    }

    #[test]
    fn article_content_is_sanitized_and_profiles_are_isolated() {
        let (mut connection, first, second) = setup();
        let feed_id = setup_feed(&mut connection, &first);
        let item_id = insert_item(&connection, &first, &feed_id);

        let detail = get_detail(&connection, &first, &item_id).unwrap();
        assert!(detail.safe_html.contains("Hello <b>reader</b>"));
        assert!(!detail.safe_html.contains("alert"));
        assert!(!detail.item.state.is_read);

        assert!(matches!(
            get_detail(&connection, &second, &item_id),
            Err(AppError::NotFound(_))
        ));
        assert!(list_articles(&connection, &second, "all", None, 10)
            .unwrap()
            .is_empty());

        update_state(
            &mut connection,
            &first,
            &item_id,
            &ArticleStatePatch {
                is_read: Some(true),
                is_saved: Some(true),
                ..ArticleStatePatch::default()
            },
        )
        .unwrap();
        assert!(list_articles(&connection, &first, "unread", None, 10)
            .unwrap()
            .is_empty());
        assert_eq!(
            list_articles(&connection, &first, "saved", None, 10)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(list_history(&connection, &first, 10).unwrap().len(), 1);
    }

    #[test]
    fn annotations_and_collections_cascade_with_the_source_item() {
        let (mut connection, first, second) = setup();
        let feed_id = setup_feed(&mut connection, &first);
        let item_id = insert_item(&connection, &first, &feed_id);

        let note = add_note(&connection, &first, &item_id, "A private note").unwrap();
        let highlight = add_highlight(&connection, &first, &item_id, "Hello", 0, 5).unwrap();
        let collection = create_collection(&connection, &first, "Research", None).unwrap();
        add_to_collection(&connection, &first, &collection.id, &item_id).unwrap();

        let detail = get_detail(&connection, &first, &item_id).unwrap();
        assert_eq!(detail.notes.len(), 1);
        assert_eq!(detail.highlights.len(), 1);
        assert_eq!(detail.collections.len(), 1);
        assert!(matches!(
            add_note(&connection, &second, &item_id, "Blocked"),
            Err(AppError::NotFound(_))
        ));

        feed_db::delete_feed(&connection, &first, &feed_id).unwrap();
        let notes: i64 = connection
            .query_row("SELECT COUNT(*) FROM article_notes", [], |row| row.get(0))
            .unwrap();
        let highlights: i64 = connection
            .query_row("SELECT COUNT(*) FROM article_highlights", [], |row| {
                row.get(0)
            })
            .unwrap();
        let collection_items: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM collection_items WHERE collection_id = ?1",
                [collection.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(notes, 0);
        assert_eq!(highlights, 0);
        assert_eq!(collection_items, 0);
        assert!(!note.id.is_empty() && !highlight.id.is_empty());
    }

    #[test]
    fn clearing_cache_keeps_the_readable_summary() {
        let (mut connection, first, _) = setup();
        let feed_id = setup_feed(&mut connection, &first);
        let item_id = insert_item(&connection, &first, &feed_id);
        assert_eq!(cache_stats(&connection, &first).unwrap().articles, 1);
        assert_eq!(
            clear_cache(&mut connection, &first, Some(&item_id)).unwrap(),
            1
        );

        let detail = get_detail(&connection, &first, &item_id).unwrap();
        assert!(detail.safe_html.contains("Plain summary"));
        assert!(!detail.safe_html.contains("Hello"));
        assert_eq!(cache_stats(&connection, &first).unwrap().images, 0);
    }
}
