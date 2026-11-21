use super::models::{SearchPresetDto, SearchResultDto, SearchStatsDto};
use super::query::{self, Filter, SearchPlan};
use crate::articles::store::{map_article, ARTICLE_COLUMNS};
use crate::error::{AppError, AppResult};
use crate::validation::normalize_display_name;
use chrono::Utc;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

fn now() -> String {
    Utc::now().to_rfc3339()
}

const PRESET_COLUMNS: &str = "id, name, query, scope, created_at, updated_at";

fn map_preset(row: &rusqlite::Row<'_>) -> rusqlite::Result<SearchPresetDto> {
    Ok(SearchPresetDto {
        id: row.get(0)?,
        name: row.get(1)?,
        query: row.get(2)?,
        scope: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn search(
    connection: &Connection,
    profile_id: &str,
    plan: &SearchPlan,
    limit: usize,
) -> AppResult<Vec<SearchResultDto>> {
    let mut clauses = vec!["search_documents.profile_id = ?".to_string()];
    let mut values: Vec<Value> = vec![Value::Text(profile_id.to_string())];

    if let Some(fts) = plan.fts.as_deref() {
        clauses.push("search_documents MATCH ?".to_string());
        values.push(Value::Text(fts.to_string()));
    }

    match plan.scope.as_str() {
        "all" => {}
        "unread" => clauses
            .push("COALESCE(s.is_read, 0) = 0 AND COALESCE(s.is_archived, 0) = 0".to_string()),
        "saved" => clauses.push("COALESCE(s.is_saved, 0) = 1".to_string()),
        "favorites" => clauses.push("COALESCE(s.is_favorite, 0) = 1".to_string()),
        "archive" => clauses.push("COALESCE(s.is_archived, 0) = 1".to_string()),
        "queue" => clauses.push("COALESCE(s.is_queued, 0) = 1".to_string()),
        _ => return Err(AppError::Validation("Unknown search scope.".to_string())),
    }

    for filter in &plan.filters {
        apply_filter(filter, &mut clauses, &mut values);
    }

    let has_fts = plan.fts.is_some();
    let projection = if has_fts {
        "snippet(search_documents, 3, char(1), char(2), '…', 24), bm25(search_documents)"
    } else {
        "'' AS snippet, 0.0 AS rank"
    };
    let order = if has_fts {
        "bm25(search_documents) ASC, COALESCE(i.published_at, i.discovered_at) DESC"
    } else {
        "COALESCE(i.published_at, i.discovered_at) DESC"
    };
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}, {projection}
         FROM search_documents
         JOIN feed_items i
           ON i.id = search_documents.item_id AND i.profile_id = search_documents.profile_id
         JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
         LEFT JOIN article_sources src
           ON src.item_id = i.id AND src.profile_id = i.profile_id
         LEFT JOIN article_states s
           ON s.item_id = i.id AND s.profile_id = i.profile_id
         WHERE {}
         ORDER BY {order}
         LIMIT ?",
        clauses.join(" AND ")
    );
    values.push(Value::Integer(limit as i64));

    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(values.iter()), |row| {
        let item = map_article(row)?;
        let snippet: String = row.get(18)?;
        let rank: f64 = row.get(19)?;
        let snippet = if snippet.trim().is_empty() {
            item.summary.clone().unwrap_or_default()
        } else {
            snippet
        };
        Ok(SearchResultDto {
            item,
            snippet,
            rank,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn apply_filter(filter: &Filter, clauses: &mut Vec<String>, values: &mut Vec<Value>) {
    match filter {
        Filter::Feed(value) => {
            clauses.push(
                "EXISTS (SELECT 1 FROM feeds fx WHERE fx.id = i.feed_id
                   AND fx.profile_id = i.profile_id AND fx.title LIKE ? COLLATE NOCASE)"
                    .to_string(),
            );
            values.push(Value::Text(format!("%{value}%")));
        }
        Filter::Folder(value) => {
            clauses.push(
                "EXISTS (SELECT 1 FROM feed_folders ff WHERE ff.id = f.folder_id
                   AND ff.profile_id = f.profile_id AND ff.name LIKE ? COLLATE NOCASE)"
                    .to_string(),
            );
            values.push(Value::Text(format!("%{value}%")));
        }
        Filter::Label(value) => {
            clauses.push(
                "EXISTS (SELECT 1 FROM label_assignments la
                   JOIN labels l ON l.id = la.label_id AND l.profile_id = la.profile_id
                   WHERE la.item_id = i.id AND la.profile_id = i.profile_id
                     AND l.name LIKE ? COLLATE NOCASE)"
                    .to_string(),
            );
            values.push(Value::Text(format!("%{value}%")));
        }
        Filter::Author(value) => {
            clauses.push("COALESCE(i.author, '') LIKE ? COLLATE NOCASE".to_string());
            values.push(Value::Text(format!("%{value}%")));
        }
        Filter::Category(value) => {
            clauses.push("COALESCE(f.category, '') LIKE ? COLLATE NOCASE".to_string());
            values.push(Value::Text(format!("%{value}%")));
        }
        Filter::Trust(value) => {
            clauses.push("f.trust_level = ?".to_string());
            values.push(Value::Text(value.clone()));
        }
        Filter::Before(value) => {
            clauses
                .push("substr(COALESCE(i.published_at, i.discovered_at), 1, 10) < ?".to_string());
            values.push(Value::Text(value.clone()));
        }
        Filter::After(value) => {
            clauses
                .push("substr(COALESCE(i.published_at, i.discovered_at), 1, 10) >= ?".to_string());
            values.push(Value::Text(value.clone()));
        }
        Filter::State { column, value } => {
            clauses.push(format!("COALESCE(s.{column}, 0) = ?"));
            values.push(Value::Integer(i64::from(*value)));
        }
        Filter::HasNotes(value) => {
            clauses.push(format!(
                "{} EXISTS (SELECT 1 FROM article_notes n
                   WHERE n.item_id = i.id AND n.profile_id = i.profile_id)",
                if *value { "" } else { "NOT" }
            ));
        }
        Filter::HasHighlights(value) => {
            clauses.push(format!(
                "{} EXISTS (SELECT 1 FROM article_highlights h
                   WHERE h.item_id = i.id AND h.profile_id = i.profile_id)",
                if *value { "" } else { "NOT" }
            ));
        }
    }
}

pub fn list_presets(connection: &Connection, profile_id: &str) -> AppResult<Vec<SearchPresetDto>> {
    let sql = format!(
        "SELECT {PRESET_COLUMNS} FROM search_presets WHERE profile_id = ?1
         ORDER BY name COLLATE NOCASE ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map([profile_id], map_preset)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_preset(
    connection: &Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<SearchPresetDto> {
    let sql =
        format!("SELECT {PRESET_COLUMNS} FROM search_presets WHERE id = ?1 AND profile_id = ?2");
    connection
        .query_row(&sql, params![id, profile_id], map_preset)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(
                "That saved search is not available in this profile.".to_string(),
            ),
            other => AppError::from(other),
        })
}

pub fn save_preset(
    connection: &Connection,
    profile_id: &str,
    id: Option<&str>,
    name: &str,
    query_text: &str,
    scope: &str,
) -> AppResult<SearchPresetDto> {
    let (name, normalized) = normalize_display_name(name, "Saved search name")?;
    let plan = query::parse(query_text, scope)?;
    let query_text = query_text.trim();
    if query_text.chars().count() > 500 {
        return Err(AppError::Validation(
            "Saved searches must be 500 characters or fewer.".to_string(),
        ));
    }

    let duplicate: Option<String> = connection
        .query_row(
            "SELECT id FROM search_presets
             WHERE profile_id = ?1 AND name_normalized = ?2 AND (?3 IS NULL OR id <> ?3)",
            params![profile_id, normalized, id],
            |row| row.get(0),
        )
        .optional()?;
    if duplicate.is_some() {
        return Err(AppError::Validation(
            "A saved search with that name already exists.".to_string(),
        ));
    }

    let timestamp = now();
    let preset_id = match id {
        Some(existing) => {
            let changed = connection.execute(
                "UPDATE search_presets SET name = ?1, name_normalized = ?2, query = ?3,
                    scope = ?4, updated_at = ?5
                 WHERE id = ?6 AND profile_id = ?7",
                params![name, normalized, query_text, plan.scope, timestamp, existing, profile_id,],
            )?;
            if changed == 0 {
                return Err(AppError::NotFound(
                    "That saved search is not available in this profile.".to_string(),
                ));
            }
            existing.to_string()
        }
        None => {
            let new_id = uuid::Uuid::new_v4().to_string();
            connection.execute(
                "INSERT INTO search_presets (
                    id, profile_id, name, name_normalized, query, scope, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                params![new_id, profile_id, name, normalized, query_text, plan.scope, timestamp,],
            )?;
            new_id
        }
    };
    get_preset(connection, profile_id, &preset_id)
}

pub fn delete_preset(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM search_presets WHERE id = ?1 AND profile_id = ?2",
        params![id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That saved search is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn stats(connection: &Connection, profile_id: &str) -> AppResult<SearchStatsDto> {
    let count = |sql: &str| -> AppResult<i64> {
        Ok(connection.query_row(sql, [profile_id], |row| row.get(0))?)
    };
    Ok(SearchStatsDto {
        indexed_documents: count("SELECT COUNT(*) FROM search_documents WHERE profile_id = ?1")?,
        presets: count("SELECT COUNT(*) FROM search_presets WHERE profile_id = ?1")?,
        story_groups: count("SELECT COUNT(*) FROM story_groups WHERE profile_id = ?1")?,
        story_items: count("SELECT COUNT(*) FROM story_group_items WHERE profile_id = ?1")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, feeds};

    fn setup() -> (Connection, String, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        db::profiles::ensure_default_profile(&mut connection).unwrap();
        let first = db::settings::get_active_profile_id(&connection).unwrap();
        let second = db::profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    fn seed(
        connection: &mut Connection,
        profile_id: &str,
        feed_id: &str,
        items: &[feeds::NewItem<'_>],
    ) {
        feeds::subscribe(
            connection,
            profile_id,
            feed_id,
            &feeds::NewFeed {
                title: "Rust Weekly",
                site_url: None,
                feed_url: &format!("https://example.com/{feed_id}.xml"),
                feed_url_normalized: &format!("https://example.com/{feed_id}.xml"),
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
        feeds::insert_items(connection, profile_id, feed_id, items).unwrap();
    }

    fn item<'a>(guid: &'a str, title: &'a str, summary: &'a str) -> feeds::NewItem<'a> {
        feeds::NewItem {
            guid,
            title,
            url: Some("https://example.com/post"),
            author: None,
            summary: Some(summary),
            content_html: None,
            published_at: None,
        }
    }

    #[test]
    fn searches_fts_content_and_keeps_profiles_isolated() {
        let (mut connection, first, second) = setup();
        seed(
            &mut connection,
            &first,
            "feed-1",
            &[
                item("a", "Rust memory safety", "Ownership prevents data races"),
                item("b", "Gardening notes", "Tomatoes need sun"),
            ],
        );
        let plan = query::parse("ownership", "all").unwrap();
        let results = search(&connection, &first, &plan, 20).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.title, "Rust memory safety");
        assert!(results[0].snippet.contains("Ownership"));

        let other = search(&connection, &second, &plan, 20).unwrap();
        assert!(other.is_empty());
    }

    #[test]
    fn filters_by_state_label_and_date() {
        let (mut connection, first, _second) = setup();
        seed(
            &mut connection,
            &first,
            "feed-1",
            &[item(
                "a",
                "Rust memory safety",
                "Ownership prevents data races",
            )],
        );
        let item_id: String = connection
            .query_row(
                "SELECT id FROM feed_items WHERE profile_id = ?1",
                [&first],
                |row| row.get(0),
            )
            .unwrap();
        crate::articles::store::update_state(
            &mut connection,
            &first,
            &item_id,
            &crate::articles::models::ArticleStatePatch {
                is_saved: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let label = crate::discovery::store::create_label(
            &connection,
            &first,
            "label-1",
            "tag",
            "Security",
            None,
            None,
        )
        .unwrap();
        crate::discovery::store::assign_label(&connection, &first, &label.id, &item_id).unwrap();

        let saved = query::parse("rust is:saved tag:Security", "all").unwrap();
        assert_eq!(search(&connection, &first, &saved, 20).unwrap().len(), 1);

        let unread = query::parse("rust is:unread", "all").unwrap();
        assert_eq!(search(&connection, &first, &unread, 20).unwrap().len(), 1);

        let read = query::parse("rust is:read", "all").unwrap();
        assert!(search(&connection, &first, &read, 20).unwrap().is_empty());
    }

    #[test]
    fn presets_are_validated_scoped_and_unique() {
        let (connection, first, second) = setup();
        let preset = save_preset(
            &connection,
            &first,
            None,
            "Security watch",
            "tag:Security",
            "unread",
        )
        .unwrap();
        assert_eq!(preset.scope, "unread");
        assert_eq!(list_presets(&connection, &first).unwrap().len(), 1);
        assert!(list_presets(&connection, &second).unwrap().is_empty());
        assert!(get_preset(&connection, &second, &preset.id).is_err());

        assert!(save_preset(&connection, &first, None, "security watch", "rust", "all").is_err());
        assert!(save_preset(&connection, &first, None, "Bad", "is:sideways", "all").is_err());

        let updated = save_preset(
            &connection,
            &first,
            Some(&preset.id),
            "Security watch",
            "rust",
            "all",
        )
        .unwrap();
        assert_eq!(updated.query, "rust");
        assert_eq!(updated.scope, "all");

        delete_preset(&connection, &first, &preset.id).unwrap();
        assert!(list_presets(&connection, &first).unwrap().is_empty());
        assert!(delete_preset(&connection, &second, &preset.id).is_err());
    }

    #[test]
    fn stats_report_real_local_counts() {
        let (mut connection, first, _second) = setup();
        seed(
            &mut connection,
            &first,
            "feed-1",
            &[item(
                "a",
                "Rust memory safety",
                "Ownership prevents data races",
            )],
        );
        let stats = stats(&connection, &first).unwrap();
        assert_eq!(stats.indexed_documents, 1);
        assert_eq!(stats.presets, 0);
        assert_eq!(stats.story_groups, 0);
    }
}
