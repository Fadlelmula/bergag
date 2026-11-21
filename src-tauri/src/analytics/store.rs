use super::models::{
    ActivityDayDto, BriefingDto, BriefingHighlightDto, FeedHealthDto, LibraryStatsDto,
    NamedCountDto, ReadingStatsDto, StorageStatsDto,
};
use crate::articles::models::ArticleListItemDto;
use crate::articles::store::{list_articles, map_article, ARTICLE_COLUMNS, ARTICLE_FROM};
use crate::error::{AppError, AppResult};
use crate::feeds::models::health_for_failures;
use crate::security::redaction::redact;
use chrono::{Duration, NaiveDate, Utc};
use rusqlite::{params, Connection};

fn day_string(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn window_start(days: i64) -> String {
    let start = Utc::now().date_naive() - Duration::days(days - 1);
    format!("{}T00:00:00Z", day_string(start))
}

pub fn reading(connection: &Connection, profile_id: &str, days: i64) -> AppResult<ReadingStatsDto> {
    let since = window_start(days);
    let mut statement = connection.prepare(
        "SELECT substr(occurred_at, 1, 10) AS day, event_type, COUNT(*)
         FROM article_history
         WHERE profile_id = ?1 AND occurred_at >= ?2
         GROUP BY day, event_type",
    )?;
    let rows = statement.query_map(params![profile_id, since], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut counts: std::collections::BTreeMap<String, (i64, i64, i64)> =
        std::collections::BTreeMap::new();
    for row in rows {
        let (day, event, count) = row?;
        let entry = counts.entry(day).or_insert((0, 0, 0));
        match event.as_str() {
            "opened" => entry.0 += count,
            "read" => entry.1 += count,
            "unread" => entry.2 += count,
            _ => {}
        }
    }

    let today = Utc::now().date_naive();
    let first_day = today - Duration::days(days - 1);
    let activity: Vec<ActivityDayDto> = (0..days)
        .map(|offset| {
            let date = first_day + Duration::days(offset);
            let key = day_string(date);
            let (opened, read, unread) = counts.get(&key).copied().unwrap_or((0, 0, 0));
            ActivityDayDto {
                date: key,
                opened,
                read,
                unread,
            }
        })
        .collect();

    let totals: (i64, i64, i64, i64, Option<String>) = connection.query_row(
        "SELECT
           (SELECT COUNT(*) FROM article_history WHERE profile_id = ?1 AND event_type = 'opened'),
           (SELECT COUNT(*) FROM article_history WHERE profile_id = ?1 AND event_type = 'read'),
           (SELECT COUNT(*) FROM article_highlights WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM article_notes WHERE profile_id = ?1),
           (SELECT MAX(last_read_at) FROM article_states WHERE profile_id = ?1)",
        [profile_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;

    let read_days: std::collections::HashSet<String> = counts
        .iter()
        .filter(|(_, (_, read, _))| *read > 0)
        .map(|(day, _)| day.clone())
        .collect();
    let mut cursor = if read_days.contains(&day_string(today)) {
        today
    } else {
        today - Duration::days(1)
    };
    let mut streak_days = 0;
    while read_days.contains(&day_string(cursor)) {
        streak_days += 1;
        cursor -= Duration::days(1);
    }

    Ok(ReadingStatsDto {
        days,
        activity,
        total_opened: totals.0,
        total_read: totals.1,
        highlights: totals.2,
        notes: totals.3,
        streak_days,
        last_read_at: totals.4,
    })
}

pub fn library(connection: &Connection, profile_id: &str) -> AppResult<LibraryStatsDto> {
    connection
        .query_row(
            "SELECT
           (SELECT COUNT(*) FROM feed_items WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM feeds WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM feed_folders WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM collections WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM labels WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM article_highlights WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM article_notes WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM story_groups WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM search_presets WHERE profile_id = ?1),
           (SELECT COALESCE(SUM(word_count), 0) FROM article_sources WHERE profile_id = ?1),
           (SELECT MIN(discovered_at) FROM feed_items WHERE profile_id = ?1),
           (SELECT MAX(discovered_at) FROM feed_items WHERE profile_id = ?1)",
            [profile_id],
            |row| {
                Ok(LibraryStatsDto {
                    articles: row.get(0)?,
                    feeds: row.get(1)?,
                    folders: row.get(2)?,
                    collections: row.get(3)?,
                    labels: row.get(4)?,
                    highlights: row.get(5)?,
                    notes: row.get(6)?,
                    stories: row.get(7)?,
                    presets: row.get(8)?,
                    words: row.get(9)?,
                    oldest_article_at: row.get(10)?,
                    newest_article_at: row.get(11)?,
                })
            },
        )
        .map_err(AppError::from)
}

pub fn storage(connection: &Connection, profile_id: &str) -> AppResult<StorageStatsDto> {
    connection
        .query_row(
            "SELECT
           (SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()),
           (SELECT COALESCE(SUM(LENGTH(data)), 0) FROM article_images WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM article_images WHERE profile_id = ?1),
           (SELECT COUNT(*) FROM article_sources WHERE profile_id = ?1),
           (SELECT COALESCE(SUM(
              LENGTH(content_text) + LENGTH(safe_html) + LENGTH(COALESCE(raw_html, ''))
            ), 0) FROM article_sources WHERE profile_id = ?1)",
            [profile_id],
            |row| {
                Ok(StorageStatsDto {
                    database_bytes: row.get(0)?,
                    cached_image_bytes: row.get(1)?,
                    cached_images: row.get(2)?,
                    extracted_articles: row.get(3)?,
                    text_characters: row.get(4)?,
                })
            },
        )
        .map_err(AppError::from)
}

pub fn feed_health(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<FeedHealthDto>> {
    let since = window_start(30);
    let mut statement = connection.prepare(
        "SELECT f.id, f.title, f.status, f.consecutive_failures,
           f.last_refreshed_at, f.last_success_at, COALESCE(f.last_error, ''),
           (SELECT COUNT(*) FROM feed_items i
             WHERE i.profile_id = f.profile_id AND i.feed_id = f.id),
           (SELECT COUNT(*) FROM feed_items i
             LEFT JOIN article_states s
               ON s.item_id = i.id AND s.profile_id = i.profile_id
             WHERE i.profile_id = f.profile_id AND i.feed_id = f.id
               AND COALESCE(s.is_read, 0) = 0 AND COALESCE(s.is_archived, 0) = 0),
           (SELECT COUNT(*) FROM feed_refresh_logs l
             WHERE l.profile_id = f.profile_id AND l.feed_id = f.id
               AND l.status = 'error' AND l.started_at >= ?2)
         FROM feeds f WHERE f.profile_id = ?1
         ORDER BY f.consecutive_failures DESC, f.title COLLATE NOCASE
         LIMIT ?3",
    )?;
    let rows = statement.query_map(params![profile_id, since, limit as i64], |row| {
        let failures: i64 = row.get(3)?;
        let last_error: String = row.get(6)?;
        Ok(FeedHealthDto {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            health: health_for_failures(failures).to_string(),
            consecutive_failures: failures,
            failures_30d: row.get(9)?,
            last_refreshed_at: row.get(4)?,
            last_success_at: row.get(5)?,
            last_error: if last_error.is_empty() {
                None
            } else {
                Some(redact(&last_error))
            },
            items: row.get(7)?,
            unread: row.get(8)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn unread_feeds(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<NamedCountDto>> {
    let mut statement = connection.prepare(
        "SELECT f.id, f.title,
           SUM(CASE WHEN COALESCE(s.is_read, 0) = 0 AND COALESCE(s.is_archived, 0) = 0
             THEN 1 ELSE 0 END) AS unread
         FROM feeds f
         JOIN feed_items i ON i.feed_id = f.id AND i.profile_id = f.profile_id
         LEFT JOIN article_states s ON s.item_id = i.id AND s.profile_id = i.profile_id
         WHERE f.profile_id = ?1
         GROUP BY f.id
         HAVING unread > 0
         ORDER BY unread DESC, f.title COLLATE NOCASE
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![profile_id, limit as i64], |row| {
        Ok(NamedCountDto {
            id: row.get(0)?,
            name: row.get(1)?,
            count: row.get(2)?,
            kind: None,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn top_labels(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<NamedCountDto>> {
    let mut statement = connection.prepare(
        "SELECT l.id, l.name, l.kind, COUNT(a.item_id) AS assignments
         FROM labels l
         LEFT JOIN label_assignments a
           ON a.label_id = l.id AND a.profile_id = l.profile_id
         WHERE l.profile_id = ?1
         GROUP BY l.id
         HAVING assignments > 0
         ORDER BY assignments DESC, l.name COLLATE NOCASE
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![profile_id, limit as i64], |row| {
        Ok(NamedCountDto {
            id: row.get(0)?,
            name: row.get(1)?,
            count: row.get(3)?,
            kind: Some(row.get::<_, String>(2)?),
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn articles(
    connection: &Connection,
    profile_id: &str,
    kind: &str,
    limit: usize,
) -> AppResult<Vec<ArticleListItemDto>> {
    match kind {
        "queue" => list_articles(connection, profile_id, "queue", None, limit),
        "recent" => {
            let sql = format!(
                "SELECT {ARTICLE_COLUMNS} FROM {ARTICLE_FROM}
                 WHERE i.profile_id = ?1
                 ORDER BY i.discovered_at DESC, i.id DESC LIMIT ?2"
            );
            let mut statement = connection.prepare(&sql)?;
            let rows = statement.query_map(params![profile_id, limit as i64], map_article)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(AppError::from)
        }
        _ => Err(AppError::Validation(
            "Article list must be recent or queue.".to_string(),
        )),
    }
}

pub fn briefing(connection: &Connection, profile_id: &str, period: &str) -> AppResult<BriefingDto> {
    let days = match period {
        "daily" => 1,
        "weekly" => 7,
        _ => {
            return Err(AppError::Validation(
                "Briefing period must be daily or weekly.".to_string(),
            ))
        }
    };
    let since = window_start(days);
    let counts: (i64, i64, i64, i64, i64) = connection.query_row(
        "SELECT
           (SELECT COUNT(*) FROM feed_items WHERE profile_id = ?1 AND discovered_at >= ?2),
           (SELECT COUNT(*) FROM article_history
             WHERE profile_id = ?1 AND event_type = 'read' AND occurred_at >= ?2),
           (SELECT COUNT(*) FROM article_states
             WHERE profile_id = ?1 AND COALESCE(is_read, 0) = 0 AND COALESCE(is_archived, 0) = 0),
           (SELECT COUNT(*) FROM article_highlights
             WHERE profile_id = ?1 AND created_at >= ?2),
           (SELECT COUNT(*) FROM article_notes
             WHERE profile_id = ?1 AND created_at >= ?2)",
        params![profile_id, since],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;

    let mut statement = connection.prepare(
        "SELECT i.id, i.title, h.text, h.created_at
         FROM article_highlights h
         JOIN feed_items i ON i.id = h.item_id AND i.profile_id = h.profile_id
         WHERE h.profile_id = ?1 AND h.created_at >= ?2
         ORDER BY h.created_at DESC LIMIT 5",
    )?;
    let highlights = statement
        .query_map(params![profile_id, since], |row| {
            Ok(BriefingHighlightDto {
                item_id: row.get(0)?,
                article_title: row.get(1)?,
                text: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(BriefingDto {
        period: period.to_string(),
        since,
        generated_at: Utc::now().to_rfc3339(),
        new_articles: counts.0,
        read_events: counts.1,
        unread_total: counts.2,
        highlights_saved: counts.3,
        notes_added: counts.4,
        top_feeds: unread_feeds(connection, profile_id, 5)?,
        recent_highlights: highlights
            .into_iter()
            .map(|mut highlight| {
                let shortened: String = highlight.text.chars().take(200).collect();
                if shortened.len() < highlight.text.len() {
                    highlight.text = format!("{shortened}…");
                }
                highlight
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, feeds, profiles};

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

    fn seed_feed(connection: &mut Connection, profile_id: &str) {
        feeds::subscribe(
            connection,
            profile_id,
            "feed-a",
            &feeds::NewFeed {
                title: "Alpha News",
                site_url: None,
                feed_url: "https://alpha.example.com/feed.xml",
                feed_url_normalized: "https://alpha.example.com/feed.xml",
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
        feeds::insert_items(
            connection,
            profile_id,
            "feed-a",
            &[feeds::NewItem {
                guid: "item-1",
                title: "A local article",
                url: Some("https://alpha.example.com/item-1"),
                author: None,
                summary: Some("Summary text"),
                content_html: None,
                published_at: Some("2026-11-20T10:00:00Z"),
            }],
        )
        .unwrap();
    }

    #[test]
    fn analytics_report_real_counts_and_respect_profile_isolation() {
        let (mut connection, first, second) = setup();
        seed_feed(&mut connection, &first);
        let item_id: String = connection
            .query_row(
                "SELECT id FROM feed_items WHERE profile_id = ?1",
                [&first],
                |row| row.get(0),
            )
            .unwrap();
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "UPDATE article_states SET is_read = 1, last_read_at = ?1
                 WHERE profile_id = ?2 AND item_id = ?3",
                params![now, first, item_id],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO article_history (id, profile_id, item_id, event_type, occurred_at)
                 VALUES ('history-1', ?1, ?2, 'read', ?3)",
                params![first, item_id, now],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO labels (
                   id, profile_id, kind, name, name_normalized, created_at, updated_at
                 ) VALUES ('label-1', ?1, 'tag', 'Research', 'research', ?2, ?2)",
                params![first, now],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO label_assignments (profile_id, label_id, item_id, created_at)
                 VALUES (?1, 'label-1', ?2, ?3)",
                params![first, item_id, now],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO article_highlights (
                   id, profile_id, item_id, text, start_offset, end_offset, created_at
                 ) VALUES ('highlight-1', ?1, ?2, 'A saved passage', 0, 15, ?3)",
                params![first, item_id, now],
            )
            .unwrap();

        let first_reading = reading(&connection, &first, 7).unwrap();
        assert_eq!(first_reading.activity.len(), 7);
        assert_eq!(first_reading.total_read, 1);
        assert_eq!(first_reading.streak_days, 1);
        assert_eq!(first_reading.highlights, 1);
        let first_library = library(&connection, &first).unwrap();
        assert_eq!(first_library.articles, 1);
        assert_eq!(first_library.labels, 1);
        assert_eq!(first_library.highlights, 1);
        assert!(!top_labels(&connection, &first, 5).unwrap().is_empty());
        assert_eq!(unread_feeds(&connection, &first, 5).unwrap().len(), 0);
        assert_eq!(feed_health(&connection, &first, 5).unwrap().len(), 1);
        assert_eq!(articles(&connection, &first, "recent", 5).unwrap().len(), 1);
        let brief = briefing(&connection, &first, "daily").unwrap();
        assert_eq!(brief.new_articles, 1);
        assert_eq!(brief.read_events, 1);
        assert_eq!(brief.recent_highlights.len(), 1);
        assert!(storage(&connection, &first).unwrap().database_bytes > 0);

        assert_eq!(library(&connection, &second).unwrap().articles, 0);
        assert!(reading(&connection, &second, 7)
            .unwrap()
            .activity
            .iter()
            .all(|day| day.read == 0));
        assert!(top_labels(&connection, &second, 5).unwrap().is_empty());
        assert!(feed_health(&connection, &second, 5).unwrap().is_empty());
        assert_eq!(
            briefing(&connection, &second, "weekly")
                .unwrap()
                .new_articles,
            0
        );
    }

    #[test]
    fn reading_returns_empty_buckets_and_validates_inputs() {
        let (connection, profile_id, _) = setup();
        let reading = reading(&connection, &profile_id, 3).unwrap();
        assert_eq!(reading.activity.len(), 3);
        assert!(reading
            .activity
            .iter()
            .all(|day| day.opened == 0 && day.read == 0));
        assert!(briefing(&connection, &profile_id, "monthly").is_err());
        assert!(articles(&connection, &profile_id, "all", 5).is_err());
    }
}
