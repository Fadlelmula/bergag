use super::models::{RelatedArticleDto, StoryDetectionDto, StoryGroupDto, StoryTimelineBucketDto};
use crate::articles::store::list_articles_by_ids;
use crate::error::{AppError, AppResult};
use chrono::{Duration, NaiveDate, Utc};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, HashMap, HashSet};

const MAX_CANDIDATES: usize = 1_000;
const TITLE_SIMILARITY: f64 = 0.72;
const STORY_WINDOW_DAYS: i64 = 45;
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "of", "to", "in", "on", "for", "with", "at", "by", "from", "is",
    "are", "was", "were", "be", "been", "this", "that", "these", "those", "it", "its", "as", "but",
    "not", "new", "says", "say", "how", "why", "what", "who", "after", "before", "over", "under",
    "about", "into", "than", "then", "their", "they", "you", "your", "we", "our", "his", "her",
    "he", "she", "i", "s", "t", "will", "can", "could", "would", "should", "has", "have", "had",
    "more", "most", "up", "out", "just", "now",
];

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn date_prefix(value: &str) -> &str {
    value.get(..10).unwrap_or(value)
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_prefix(value), "%Y-%m-%d").ok()
}

fn within_window(left: &str, right: &str, days: i64) -> bool {
    match (parse_date(left), parse_date(right)) {
        (Some(left), Some(right)) => (left - right).num_days().abs() <= days,
        _ => true,
    }
}

fn tokens(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= 2 && !STOPWORDS.contains(token))
        .map(str::to_string)
        .collect()
}

fn similarity(left: &HashSet<String>, right: &HashSet<String>) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let shared = left.intersection(right).count();
    if shared < 3 {
        return 0.0;
    }
    shared as f64 / left.union(right).count() as f64
}

fn is_tracking_parameter(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.starts_with("utm_")
        || matches!(
            key.as_str(),
            "fbclid" | "gclid" | "mc_cid" | "mc_eid" | "igshid" | "ref_src"
        )
}

fn canonical_url(raw: &str) -> Option<String> {
    let mut url = url::Url::parse(raw).ok()?;
    url.set_fragment(None);
    let host = url.host_str()?.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();
    url.set_host(Some(&host)).ok()?;

    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| !is_tracking_parameter(key))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in &pairs {
        serializer.append_pair(key, value);
    }
    let query = serializer.finish();
    url.set_query(if query.is_empty() { None } else { Some(&query) });

    let path = url.path().trim_end_matches('/').to_string();
    url.set_path(if path.is_empty() { "/" } else { &path });
    Some(url.to_string())
}

struct Candidate {
    id: String,
    title: String,
    published: String,
    canonical: Option<String>,
    tokens: HashSet<String>,
}

struct GroupState {
    id: String,
    latest: String,
    canonical: HashSet<String>,
    tokens: HashSet<String>,
}

pub fn detect_duplicates(
    connection: &mut Connection,
    profile_id: &str,
) -> AppResult<StoryDetectionDto> {
    let transaction = connection.transaction()?;

    let mut groups: Vec<GroupState> = Vec::new();
    {
        let mut statement = transaction.prepare(
            "SELECT g.id, g.title,
                COALESCE(MAX(COALESCE(i.published_at, i.discovered_at)), g.created_at)
             FROM story_groups g
             LEFT JOIN story_group_items sgi
               ON sgi.story_id = g.id AND sgi.profile_id = g.profile_id
             LEFT JOIN feed_items i
               ON i.id = sgi.item_id AND i.profile_id = sgi.profile_id
             WHERE g.profile_id = ?1
             GROUP BY g.id
             ORDER BY 3 DESC
             LIMIT 500",
        )?;
        let rows = statement
            .query_map([profile_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (id, _title, latest) in rows {
            let mut canonical = HashSet::new();
            let mut title_tokens = HashSet::new();
            let mut item_statement = transaction.prepare(
                "SELECT i.title, i.url FROM story_group_items sgi
                 JOIN feed_items i ON i.id = sgi.item_id AND i.profile_id = sgi.profile_id
                 WHERE sgi.story_id = ?1 AND sgi.profile_id = ?2 LIMIT 20",
            )?;
            let items = item_statement
                .query_map(params![id, profile_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            for (title, url) in items {
                title_tokens.extend(tokens(&title));
                if let Some(url) = url.as_deref().and_then(canonical_url) {
                    canonical.insert(url);
                }
            }
            groups.push(GroupState {
                id,
                latest,
                canonical,
                tokens: title_tokens,
            });
        }
    }

    let candidates: Vec<Candidate> = {
        let mut statement = transaction.prepare(
            "SELECT i.id, i.title, i.url, COALESCE(i.published_at, i.discovered_at)
             FROM feed_items i
             LEFT JOIN story_group_items sgi
               ON sgi.item_id = i.id AND sgi.profile_id = i.profile_id
             WHERE i.profile_id = ?1 AND sgi.item_id IS NULL
             ORDER BY COALESCE(i.published_at, i.discovered_at) DESC
             LIMIT ?2",
        )?;
        let candidates = statement
            .query_map(params![profile_id, MAX_CANDIDATES as i64], |row| {
                let title: String = row.get(1)?;
                Ok(Candidate {
                    id: row.get(0)?,
                    tokens: tokens(&title),
                    title,
                    published: row.get(3)?,
                    canonical: row
                        .get::<_, Option<String>>(2)?
                        .as_deref()
                        .and_then(canonical_url),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        candidates
    };

    let timestamp = now();
    let mut groups_created = 0i64;
    let mut items_grouped = 0i64;

    for candidate in candidates {
        let mut best: Option<(usize, f64)> = None;
        for (index, group) in groups.iter().enumerate() {
            if !within_window(&candidate.published, &group.latest, STORY_WINDOW_DAYS) {
                continue;
            }
            let score = match candidate.canonical.as_deref() {
                Some(url) if group.canonical.contains(url) => 1.0,
                _ => similarity(&candidate.tokens, &group.tokens),
            };
            if score >= TITLE_SIMILARITY && best.map_or(true, |(_, current)| score > current) {
                best = Some((index, score));
            }
        }

        if let Some((index, score)) = best {
            let group_id = groups[index].id.clone();
            transaction.execute(
                "INSERT OR IGNORE INTO story_group_items (
                    story_id, item_id, profile_id, similarity, added_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![group_id, candidate.id, profile_id, score, timestamp],
            )?;
            groups[index].tokens.extend(candidate.tokens);
            if let Some(url) = candidate.canonical {
                groups[index].canonical.insert(url);
            }
            if candidate.published > groups[index].latest {
                groups[index].latest = candidate.published;
            }
            items_grouped += 1;
            continue;
        }

        let group_id = uuid::Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO story_groups (
                id, profile_id, title, canonical_item_id, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![
                group_id,
                profile_id,
                candidate.title,
                candidate.id,
                timestamp
            ],
        )?;
        transaction.execute(
            "INSERT INTO story_group_items (
                story_id, item_id, profile_id, similarity, added_at
             ) VALUES (?1, ?2, ?3, 1.0, ?4)",
            params![group_id, candidate.id, profile_id, timestamp],
        )?;
        groups.push(GroupState {
            id: group_id,
            latest: candidate.published,
            canonical: candidate.canonical.into_iter().collect(),
            tokens: candidate.tokens,
        });
        groups_created += 1;
        items_grouped += 1;
    }

    transaction.commit()?;
    Ok(StoryDetectionDto {
        groups_created,
        items_grouped,
    })
}

fn preview_item_ids(
    connection: &Connection,
    profile_id: &str,
    story_id: &str,
    limit: usize,
) -> AppResult<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT sgi.item_id FROM story_group_items sgi
         JOIN feed_items i ON i.id = sgi.item_id AND i.profile_id = sgi.profile_id
         WHERE sgi.story_id = ?1 AND sgi.profile_id = ?2
         ORDER BY COALESCE(i.published_at, i.discovered_at) DESC
         LIMIT ?3",
    )?;
    let ids = statement
        .query_map(params![story_id, profile_id, limit as i64], |row| {
            row.get(0)
        })?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(ids)
}

fn list_groups(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
    since: Option<&str>,
    preview: usize,
) -> AppResult<Vec<StoryGroupDto>> {
    let mut statement = connection.prepare(
        "SELECT g.id, g.title, g.created_at,
            (SELECT COUNT(*) FROM story_group_items x
              WHERE x.story_id = g.id AND x.profile_id = g.profile_id),
            (SELECT COUNT(DISTINCT i.feed_id) FROM story_group_items x
              JOIN feed_items i ON i.id = x.item_id AND i.profile_id = x.profile_id
              WHERE x.story_id = g.id AND x.profile_id = g.profile_id),
            (SELECT MAX(COALESCE(i.published_at, i.discovered_at))
              FROM story_group_items x
              JOIN feed_items i ON i.id = x.item_id AND i.profile_id = x.profile_id
              WHERE x.story_id = g.id AND x.profile_id = g.profile_id)
         FROM story_groups g
         WHERE g.profile_id = ?1
           AND (?2 IS NULL OR EXISTS (
             SELECT 1 FROM story_group_items x
             JOIN feed_items i ON i.id = x.item_id AND i.profile_id = x.profile_id
             WHERE x.story_id = g.id AND x.profile_id = g.profile_id
               AND COALESCE(i.published_at, i.discovered_at) >= ?2
           ))
         ORDER BY 6 DESC
         LIMIT ?3",
    )?;
    let rows = statement
        .query_map(params![profile_id, since, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut groups = Vec::with_capacity(rows.len());
    for (id, title, created_at, item_count, source_count, latest_at) in rows {
        let ids = preview_item_ids(connection, profile_id, &id, preview)?;
        let items = list_articles_by_ids(connection, profile_id, &ids, preview.max(1))?;
        groups.push(StoryGroupDto {
            id,
            title,
            item_count,
            source_count,
            latest_at,
            created_at,
            items,
        });
    }
    Ok(groups)
}

pub fn list_stories(
    connection: &Connection,
    profile_id: &str,
    limit: usize,
) -> AppResult<Vec<StoryGroupDto>> {
    list_groups(connection, profile_id, limit, None, 4)
}

pub fn get_story(
    connection: &Connection,
    profile_id: &str,
    story_id: &str,
) -> AppResult<StoryGroupDto> {
    let mut groups = list_groups(connection, profile_id, 1, None, 200)?;
    if let Some(index) = groups.iter().position(|group| group.id == story_id) {
        return Ok(groups.remove(index));
    }

    // The limit above orders by latest activity, so a direct lookup is needed
    // when the requested story is older than the newest one.
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM story_groups WHERE id = ?1 AND profile_id = ?2)",
        params![story_id, profile_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound(
            "That story group is not available in this profile.".to_string(),
        ));
    }
    let ids = preview_item_ids(connection, profile_id, story_id, 200)?;
    let items = list_articles_by_ids(connection, profile_id, &ids, 200)?;
    connection
        .query_row(
            "SELECT g.id, g.title, g.created_at,
                (SELECT COUNT(*) FROM story_group_items x
                  WHERE x.story_id = g.id AND x.profile_id = g.profile_id),
                (SELECT COUNT(DISTINCT i.feed_id) FROM story_group_items x
                  JOIN feed_items i ON i.id = x.item_id AND i.profile_id = x.profile_id
                  WHERE x.story_id = g.id AND x.profile_id = g.profile_id),
                (SELECT MAX(COALESCE(i.published_at, i.discovered_at))
                  FROM story_group_items x
                  JOIN feed_items i ON i.id = x.item_id AND i.profile_id = x.profile_id
                  WHERE x.story_id = g.id AND x.profile_id = g.profile_id)
             FROM story_groups g WHERE g.id = ?1 AND g.profile_id = ?2",
            params![story_id, profile_id],
            |row| {
                Ok(StoryGroupDto {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    item_count: row.get(3)?,
                    source_count: row.get(4)?,
                    latest_at: row.get(5)?,
                    items: items.clone(),
                })
            },
        )
        .map_err(AppError::from)
}

pub fn timeline(
    connection: &Connection,
    profile_id: &str,
    days: i64,
    limit: usize,
) -> AppResult<Vec<StoryTimelineBucketDto>> {
    let since = (Utc::now() - Duration::days(days)).to_rfc3339();
    let groups = list_groups(connection, profile_id, limit, Some(&since), 3)?;
    let mut buckets: BTreeMap<String, Vec<StoryGroupDto>> = BTreeMap::new();
    for group in groups {
        let date = group
            .latest_at
            .as_deref()
            .map(date_prefix)
            .unwrap_or("Undated")
            .to_string();
        buckets.entry(date).or_default().push(group);
    }
    Ok(buckets
        .into_iter()
        .rev()
        .map(|(date, stories)| StoryTimelineBucketDto { date, stories })
        .collect())
}

pub fn delete_story(connection: &Connection, profile_id: &str, story_id: &str) -> AppResult<()> {
    let changed = connection.execute(
        "DELETE FROM story_groups WHERE id = ?1 AND profile_id = ?2",
        params![story_id, profile_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(
            "That story group is not available in this profile.".to_string(),
        ));
    }
    Ok(())
}

pub fn related_articles(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    limit: usize,
) -> AppResult<Vec<RelatedArticleDto>> {
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

    let mut scores: HashMap<String, (i64, Vec<String>)> = HashMap::new();

    {
        let mut statement = connection.prepare(
            "SELECT sgi2.item_id FROM story_group_items sgi1
             JOIN story_group_items sgi2
               ON sgi2.story_id = sgi1.story_id AND sgi2.profile_id = sgi1.profile_id
             WHERE sgi1.item_id = ?1 AND sgi1.profile_id = ?2 AND sgi2.item_id <> ?1
             LIMIT 50",
        )?;
        let ids = statement
            .query_map(params![item_id, profile_id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for id in ids {
            let entry = scores.entry(id).or_insert((0, Vec::new()));
            entry.0 += 100;
            entry.1.push("Same story".to_string());
        }
    }

    {
        let mut statement = connection.prepare(
            "SELECT la2.item_id, COUNT(*) AS shared, GROUP_CONCAT(l.name, char(31))
             FROM label_assignments la1
             JOIN label_assignments la2
               ON la2.label_id = la1.label_id AND la2.profile_id = la1.profile_id
               AND la2.item_id <> la1.item_id
             JOIN labels l ON l.id = la1.label_id AND l.profile_id = la1.profile_id
             WHERE la1.item_id = ?1 AND la1.profile_id = ?2
             GROUP BY la2.item_id
             ORDER BY shared DESC
             LIMIT 50",
        )?;
        let rows = statement
            .query_map(params![item_id, profile_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (id, shared, labels) in rows {
            let entry = scores.entry(id).or_insert((0, Vec::new()));
            entry.0 += shared * 10;
            if let Some(labels) = labels {
                for label in labels.split('\u{1f}') {
                    let reason = format!("Shared label: {label}");
                    if !entry.1.contains(&reason) {
                        entry.1.push(reason);
                    }
                }
            }
        }
    }

    let mut ranked: Vec<(String, i64, Vec<String>)> = scores
        .into_iter()
        .map(|(id, (score, reasons))| (id, score, reasons))
        .collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked.truncate(limit.clamp(1, 20));

    let ids: Vec<String> = ranked.iter().map(|(id, _, _)| id.clone()).collect();
    let items = list_articles_by_ids(connection, profile_id, &ids, ids.len().max(1))?;
    let by_id: HashMap<String, _> = items
        .into_iter()
        .map(|item| (item.id.clone(), item))
        .collect();
    Ok(ranked
        .into_iter()
        .filter_map(|(id, score, reasons)| {
            by_id.get(&id).cloned().map(|item| RelatedArticleDto {
                item,
                score,
                reasons,
            })
        })
        .collect())
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

    fn seed_feed(
        connection: &mut Connection,
        profile_id: &str,
        feed_id: &str,
        title: &str,
        items: &[feeds::NewItem<'_>],
    ) {
        feeds::subscribe(
            connection,
            profile_id,
            feed_id,
            &feeds::NewFeed {
                title,
                site_url: None,
                feed_url: &format!("https://{feed_id}.example.com/feed.xml"),
                feed_url_normalized: &format!("https://{feed_id}.example.com/feed.xml"),
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

    fn item<'a>(guid: &'a str, title: &'a str, url: &'a str) -> feeds::NewItem<'a> {
        feeds::NewItem {
            guid,
            title,
            url: Some(url),
            author: None,
            summary: Some(title),
            content_html: None,
            published_at: Some("2026-11-19T10:00:00Z"),
        }
    }

    fn item_id(connection: &Connection, profile_id: &str, guid: &str) -> String {
        connection
            .query_row(
                "SELECT id FROM feed_items WHERE profile_id = ?1 AND guid = ?2",
                params![profile_id, guid],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn groups_duplicate_coverage_and_is_profile_scoped() {
        let (mut connection, first, second) = setup();
        seed_feed(
            &mut connection,
            &first,
            "feed-a",
            "Alpha News",
            &[item(
                "a1",
                "Rust 1.90 released with faster builds",
                "https://alpha.example.com/rust?utm_source=newsletter",
            )],
        );
        seed_feed(
            &mut connection,
            &first,
            "feed-b",
            "Beta Daily",
            &[
                item(
                    "b1",
                    "Rust 1.90 released with faster builds",
                    "https://beta.example.com/rust-1-90",
                ),
                item(
                    "b2",
                    "A guide to winter gardening",
                    "https://beta.example.com/garden",
                ),
            ],
        );

        let detection = detect_duplicates(&mut connection, &first).unwrap();
        assert_eq!(detection.groups_created, 2);
        assert_eq!(detection.items_grouped, 3);

        let stories = list_stories(&connection, &first, 20).unwrap();
        let rust = stories
            .iter()
            .find(|story| story.title.contains("Rust 1.90"))
            .expect("rust story");
        assert_eq!(rust.item_count, 2);
        assert_eq!(rust.source_count, 2);
        assert_eq!(rust.items.len(), 2);

        let again = detect_duplicates(&mut connection, &first).unwrap();
        assert_eq!(again.items_grouped, 0);
        assert!(list_stories(&connection, &second, 20).unwrap().is_empty());

        let detail = get_story(&connection, &first, &rust.id).unwrap();
        assert_eq!(detail.item_count, 2);
        assert!(get_story(&connection, &second, &rust.id).is_err());

        let buckets = timeline(&connection, &first, 90, 20).unwrap();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].date, "2026-11-19");

        delete_story(&connection, &first, &rust.id).unwrap();
        assert!(get_story(&connection, &first, &rust.id).is_err());
    }

    #[test]
    fn related_articles_use_story_and_label_connections() {
        let (mut connection, first, second) = setup();
        seed_feed(
            &mut connection,
            &first,
            "feed-a",
            "Alpha News",
            &[
                item("a1", "Rust memory safety", "https://alpha.example.com/a1"),
                item("a2", "Ownership explained", "https://alpha.example.com/a2"),
                item("a3", "Gardening in winter", "https://alpha.example.com/a3"),
            ],
        );
        let a1 = item_id(&connection, &first, "a1");
        let a2 = item_id(&connection, &first, "a2");

        let label = crate::discovery::store::create_label(
            &connection,
            &first,
            "label-1",
            "tag",
            "Rust",
            None,
            None,
        )
        .unwrap();
        crate::discovery::store::assign_label(&connection, &first, &label.id, &a1).unwrap();
        crate::discovery::store::assign_label(&connection, &first, &label.id, &a2).unwrap();

        let related = related_articles(&connection, &first, &a1, 10).unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].item.id, a2);
        assert!(related[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Rust")));

        assert!(related_articles(&connection, &second, &a1, 10).is_err());
        assert!(related_articles(&connection, &first, "missing", 10).is_err());
    }
}
