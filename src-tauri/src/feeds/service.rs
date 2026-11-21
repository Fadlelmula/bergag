use super::models::{
    DiscoveredFeedDto, FeedDto, FeedItemDto, FeedPreviewDto, OpmlImportResultDto,
    RefreshOutcomeDto, SubscribeResultDto,
};
use super::network::{self, FetchResult};
use super::parser::{self, ParsedFeed, ParsedItem};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use crate::state::AppState;
use chrono::Utc;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use reqwest::StatusCode;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const REFRESH_BATCH: usize = 20;
const REFRESH_CONCURRENCY: usize = 3;
pub const MAX_STORED_FEED_ITEMS: usize = 200;
const MAX_OPML_BYTES: usize = 2 * 1024 * 1024;
const MAX_OPML_FEEDS: usize = 500;

struct FetchedFeed {
    status: StatusCode,
    etag: Option<String>,
    last_modified: Option<String>,
    parsed: Option<ParsedFeed>,
    error: Option<String>,
    duration_ms: i64,
    started_at: String,
}

struct FetchRequest {
    feed_id: String,
    url: String,
    etag: Option<String>,
    last_modified: Option<String>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn duration_ms(start: &std::time::Instant) -> i64 {
    start.elapsed().as_millis().min(i64::MAX as u128) as i64
}

async fn fetch_feed(request: FetchRequest) -> (String, FetchedFeed) {
    let feed_id = request.feed_id.clone();
    let started = std::time::Instant::now();
    let started_at = now();
    let make = |status: StatusCode,
                etag: Option<String>,
                last_modified: Option<String>,
                parsed: Option<ParsedFeed>,
                error: Option<String>| FetchedFeed {
        status,
        etag,
        last_modified,
        parsed,
        error,
        duration_ms: duration_ms(&started),
        started_at: started_at.clone(),
    };

    let url = match network::parse_public_url(&request.url) {
        Ok(url) => url,
        Err(error) => {
            return (
                feed_id,
                make(
                    StatusCode::OK,
                    None,
                    None,
                    None,
                    Some(redact(&error.user_message())),
                ),
            )
        }
    };

    let result = match network::fetch_bytes(
        &url,
        request.etag.as_deref(),
        request.last_modified.as_deref(),
        network::MAX_FEED_BYTES,
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            return (
                feed_id,
                make(
                    StatusCode::OK,
                    None,
                    None,
                    None,
                    Some(redact(&error.user_message())),
                ),
            )
        }
    };

    let FetchResult {
        status,
        etag,
        last_modified,
        content_type: _,
        bytes,
    } = result;
    if status == StatusCode::NOT_MODIFIED {
        return (feed_id, make(status, etag, last_modified, None, None));
    }
    if !status.is_success() {
        return (
            feed_id,
            make(
                status,
                etag,
                last_modified,
                None,
                Some(format!(
                    "The feed server returned HTTP {}.",
                    status.as_u16()
                )),
            ),
        );
    }
    match parser::parse_feed_bytes(&bytes, &request.url) {
        Ok(parsed) => (
            feed_id,
            make(status, etag, last_modified, Some(parsed), None),
        ),
        Err(error) => (
            feed_id,
            make(
                status,
                etag,
                last_modified,
                None,
                Some(redact(&error.user_message())),
            ),
        ),
    }
}

fn parsed_items(parsed: &ParsedFeed) -> Vec<db::feeds::NewItem<'_>> {
    parsed
        .items
        .iter()
        .take(MAX_STORED_FEED_ITEMS)
        .map(|item: &ParsedItem| db::feeds::NewItem {
            guid: &item.guid,
            title: &item.title,
            url: item.url.as_deref(),
            author: item.author.as_deref(),
            summary: item.summary.as_deref(),
            content_html: item.content_html.as_deref(),
            published_at: item.published_at.as_deref(),
        })
        .collect()
}

pub async fn preview(raw_url: &str) -> AppResult<FeedPreviewDto> {
    let url = network::parse_public_url(raw_url)?;
    let result = network::fetch_bytes(&url, None, None, network::MAX_FEED_BYTES).await?;
    if !result.status.is_success() {
        return Err(AppError::Validation(format!(
            "The feed server returned HTTP {}.",
            result.status.as_u16()
        )));
    }
    let parsed = parser::parse_feed_bytes(&result.bytes, &url.to_string())?;
    let items = parsed
        .items
        .iter()
        .take(10)
        .map(|item| FeedItemDto {
            id: String::new(),
            feed_id: String::new(),
            guid: item.guid.clone(),
            title: item.title.clone(),
            url: item.url.clone(),
            author: item.author.clone(),
            summary: item.summary.clone(),
            published_at: item.published_at.clone(),
            discovered_at: String::new(),
        })
        .collect();
    Ok(FeedPreviewDto {
        title: parsed.title,
        site_url: parsed.site_url,
        feed_type: parsed.feed_type,
        description: parsed.description,
        language: parsed.language,
        item_count: parsed.items.len() as i64,
        items,
    })
}

pub async fn discover(raw_url: &str) -> AppResult<Vec<DiscoveredFeedDto>> {
    let url = network::parse_public_url(raw_url)?;
    let result = network::fetch_bytes(&url, None, None, network::MAX_DISCOVERY_BYTES).await?;
    if !result.status.is_success() {
        return Err(AppError::Validation(format!(
            "The website returned HTTP {}.",
            result.status.as_u16()
        )));
    }
    parser::discover_feeds(&result.bytes, &url.to_string())
}

pub async fn subscribe(
    state: &AppState,
    raw_url: &str,
    folder_id: Option<&str>,
    title_override: Option<&str>,
) -> AppResult<SubscribeResultDto> {
    let url = network::parse_public_url(raw_url)?;
    let normalized = network::normalized_url(&url);

    let existing = {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        db::feeds::find_feed_by_normalized_url(&connection, &profile_id, &normalized)?
    };

    if let Some(feed) = existing {
        let preview = match preview(&normalized).await {
            Ok(preview) => preview,
            Err(_) => FeedPreviewDto {
                title: feed.title.clone(),
                site_url: feed.site_url.clone(),
                feed_type: feed.feed_type.clone(),
                description: feed.description.clone(),
                language: feed.language.clone(),
                item_count: feed.article_count,
                items: Vec::new(),
            },
        };
        return Ok(SubscribeResultDto {
            feed,
            preview,
            duplicate: true,
        });
    }

    let result = network::fetch_bytes(&url, None, None, network::MAX_FEED_BYTES).await?;
    if !result.status.is_success() {
        return Err(AppError::Validation(format!(
            "The feed server returned HTTP {}.",
            result.status.as_u16()
        )));
    }
    let parsed = parser::parse_feed_bytes(&result.bytes, &normalized)?;
    let items = parsed_items(&parsed);
    let final_title = title_override
        .map(str::trim)
        .filter(|title| !title.is_empty() && title.chars().count() <= 256)
        .unwrap_or(&parsed.title)
        .to_string();
    let preview = FeedPreviewDto {
        title: final_title.clone(),
        site_url: parsed.site_url.clone(),
        feed_type: parsed.feed_type.clone(),
        description: parsed.description.clone(),
        language: parsed.language.clone(),
        item_count: parsed.items.len() as i64,
        items: Vec::new(),
    };

    let mut connection = state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    if let Some(feed) =
        db::feeds::find_feed_by_normalized_url(&connection, &profile_id, &normalized)?
    {
        return Ok(SubscribeResultDto {
            feed,
            preview,
            duplicate: true,
        });
    }

    let id = uuid::Uuid::new_v4().to_string();
    let new_feed = db::feeds::NewFeed {
        title: &final_title,
        site_url: parsed.site_url.as_deref(),
        feed_url: &normalized,
        feed_url_normalized: &normalized,
        feed_type: &parsed.feed_type,
        description: parsed.description.as_deref(),
        language: parsed.language.as_deref(),
        etag: result.etag.as_deref(),
        last_modified: result.last_modified.as_deref(),
        folder_id,
    };
    let (feed, _) = db::feeds::subscribe(&mut connection, &profile_id, &id, &new_feed, &items)?;
    Ok(SubscribeResultDto {
        feed,
        preview,
        duplicate: false,
    })
}

fn build_requests(
    connection: &rusqlite::Connection,
    refs: Vec<db::feeds::FeedRef>,
) -> AppResult<Vec<(String, String, FetchRequest)>> {
    let mut requests = Vec::new();
    for reference in refs {
        let data =
            db::feeds::get_feed_fetch_data(connection, &reference.profile_id, &reference.feed_id)?;
        requests.push((
            reference.profile_id,
            reference.feed_id.clone(),
            FetchRequest {
                feed_id: reference.feed_id,
                url: data.feed_url,
                etag: data.etag,
                last_modified: data.last_modified,
            },
        ));
    }
    Ok(requests)
}

pub async fn refresh(state: &AppState, feed_id: Option<&str>) -> AppResult<Vec<RefreshOutcomeDto>> {
    let requests = {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        let profile_id = db::settings::get_active_profile_id(&connection)?;
        let refs = match feed_id {
            Some(id) => vec![db::feeds::FeedRef {
                profile_id: profile_id.clone(),
                feed_id: id.to_string(),
            }],
            None => db::feeds::due_feeds_for_profile(&connection, &profile_id, REFRESH_BATCH)?,
        };
        build_requests(&connection, refs)?
    };
    run_refresh(state, requests).await
}

pub async fn refresh_due(state: &AppState) -> AppResult<Vec<RefreshOutcomeDto>> {
    let requests = {
        let connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        build_requests(
            &connection,
            db::feeds::due_feeds(&connection, REFRESH_BATCH)?,
        )?
    };
    run_refresh(state, requests).await
}

async fn run_refresh(
    state: &AppState,
    requests: Vec<(String, String, FetchRequest)>,
) -> AppResult<Vec<RefreshOutcomeDto>> {
    let semaphore = Arc::new(Semaphore::new(REFRESH_CONCURRENCY));
    let mut set = JoinSet::new();
    for (profile_id, id, request) in requests {
        let permit = semaphore.clone();
        set.spawn(async move {
            let _guard = permit.acquire().await;
            (profile_id, id, fetch_feed(request).await.1)
        });
    }

    let mut fetched = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(result) => fetched.push(result),
            Err(error) => {
                tracing::warn!(error = %redact(&error.to_string()), "feed refresh task failed")
            }
        }
    }

    let mut outcomes = Vec::new();
    for (profile_id, id, result) in fetched {
        let mut connection = state.connection.lock().map_err(|_| {
            AppError::Internal("The local database is temporarily unavailable.".to_string())
        })?;
        let data = match db::feeds::get_feed_fetch_data(&connection, &profile_id, &id) {
            Ok(data) => data,
            Err(_) => continue,
        };
        let failures = db::feeds::get_feed(&connection, &profile_id, &id)
            .map(|feed| feed.consecutive_failures)
            .unwrap_or(0);
        let interval = data.refresh_interval_minutes;
        let title = data.title;

        if let Some(error) = result.error.clone() {
            db::feeds::record_failure(
                &mut connection,
                &profile_id,
                db::feeds::RefreshFailure {
                    feed_id: &id,
                    interval_minutes: interval,
                    failures,
                    http_status: Some(result.status.as_u16() as i64),
                    duration_ms: result.duration_ms,
                    started_at: &result.started_at,
                    message: &error,
                },
            )?;
            outcomes.push(RefreshOutcomeDto {
                feed_id: id,
                title,
                status: "error".to_string(),
                http_status: Some(result.status.as_u16() as i64),
                item_count: 0,
                new_item_count: 0,
                duration_ms: result.duration_ms,
                message: Some(error),
            });
            continue;
        }

        if result.status == StatusCode::NOT_MODIFIED {
            db::feeds::record_not_modified(
                &mut connection,
                &profile_id,
                db::feeds::RefreshNotModified {
                    feed_id: &id,
                    interval_minutes: interval,
                    http_status: Some(result.status.as_u16() as i64),
                    duration_ms: result.duration_ms,
                    started_at: &result.started_at,
                },
            )?;
            outcomes.push(RefreshOutcomeDto {
                feed_id: id,
                title,
                status: "not_modified".to_string(),
                http_status: Some(result.status.as_u16() as i64),
                item_count: 0,
                new_item_count: 0,
                duration_ms: result.duration_ms,
                message: None,
            });
            continue;
        }

        if let Some(parsed) = result.parsed.as_ref() {
            let items = parsed_items(parsed);
            db::feeds::record_success(
                &mut connection,
                &profile_id,
                db::feeds::RefreshSuccess {
                    feed_id: &id,
                    etag: result.etag.as_deref(),
                    last_modified: result.last_modified.as_deref(),
                    interval_minutes: interval,
                    items: &items,
                    item_count: parsed.items.len() as i64,
                    http_status: Some(result.status.as_u16() as i64),
                    duration_ms: result.duration_ms,
                    started_at: &result.started_at,
                },
            )?;
            let logs = db::feeds::list_refresh_logs(&connection, &profile_id, &id, 1)?;
            let new_count = logs.first().map(|log| log.new_item_count).unwrap_or(0);
            outcomes.push(RefreshOutcomeDto {
                feed_id: id,
                title,
                status: "success".to_string(),
                http_status: Some(result.status.as_u16() as i64),
                item_count: parsed.items.len() as i64,
                new_item_count: new_count,
                duration_ms: result.duration_ms,
                message: None,
            });
            continue;
        }

        let message = "The feed could not be parsed.";
        db::feeds::record_failure(
            &mut connection,
            &profile_id,
            db::feeds::RefreshFailure {
                feed_id: &id,
                interval_minutes: interval,
                failures,
                http_status: Some(result.status.as_u16() as i64),
                duration_ms: result.duration_ms,
                started_at: &result.started_at,
                message,
            },
        )?;
        outcomes.push(RefreshOutcomeDto {
            feed_id: id,
            title,
            status: "error".to_string(),
            http_status: Some(result.status.as_u16() as i64),
            item_count: 0,
            new_item_count: 0,
            duration_ms: result.duration_ms,
            message: Some(message.to_string()),
        });
    }

    Ok(outcomes)
}

fn attribute(element: &BytesStart<'_>, name: &str) -> Option<String> {
    element
        .try_get_attribute(name)
        .ok()
        .flatten()
        .map(|attribute| attribute.value.trim().to_string())
        .filter(|value| !value.is_empty())
}

struct OpmlFeed {
    url: String,
    title: Option<String>,
    folder: Option<String>,
}

fn parse_opml(xml: &str) -> AppResult<Vec<OpmlFeed>> {
    if xml.len() > MAX_OPML_BYTES {
        return Err(AppError::Validation(
            "The OPML file is too large (maximum 2 MB).".to_string(),
        ));
    }
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut folders: Vec<String> = Vec::new();
    let mut open_outlines: Vec<bool> = Vec::new();
    let mut feeds = Vec::new();
    let mut seen = HashSet::new();
    let mut saw_opml_start = false;
    let mut saw_opml_end = false;

    let mut add_feed = |feed: OpmlFeed, feeds: &mut Vec<OpmlFeed>| -> AppResult<()> {
        if feeds.len() >= MAX_OPML_FEEDS {
            return Err(AppError::Validation(format!(
                "The OPML file contains too many feeds (maximum {MAX_OPML_FEEDS})."
            )));
        }
        if seen.insert(feed.url.clone()) {
            feeds.push(feed);
        }
        Ok(())
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) if element.name().as_ref() == "outline" => {
                let xml_url = attribute(&element, "xmlUrl");
                if let Some(url) = xml_url {
                    add_feed(
                        OpmlFeed {
                            url,
                            title: attribute(&element, "title")
                                .or_else(|| attribute(&element, "text")),
                            folder: folders.last().cloned(),
                        },
                        &mut feeds,
                    )?;
                    open_outlines.push(false);
                } else {
                    let name = attribute(&element, "title")
                        .or_else(|| attribute(&element, "text"))
                        .unwrap_or_else(|| "Imported folder".to_string());
                    folders.push(name);
                    open_outlines.push(true);
                }
            }
            Ok(Event::Empty(element)) if element.name().as_ref() == "outline" => {
                if let Some(url) = attribute(&element, "xmlUrl") {
                    add_feed(
                        OpmlFeed {
                            url,
                            title: attribute(&element, "title")
                                .or_else(|| attribute(&element, "text")),
                            folder: folders.last().cloned(),
                        },
                        &mut feeds,
                    )?;
                }
            }
            Ok(Event::Start(element)) if element.name().as_ref() == "opml" => {
                saw_opml_start = true;
            }
            Ok(Event::End(element)) if element.name().as_ref() == "opml" => {
                saw_opml_end = true;
            }
            Ok(Event::End(element)) if element.name().as_ref() == "outline" => {
                if open_outlines.pop().unwrap_or(false) {
                    folders.pop();
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(error = %redact(&error.to_string()), "OPML parse failed");
                return Err(AppError::Validation(
                    "The OPML file is not valid XML.".to_string(),
                ));
            }
        }
    }

    if !saw_opml_start || !saw_opml_end {
        return Err(AppError::Validation(
            "The OPML file is not valid XML.".to_string(),
        ));
    }

    Ok(feeds)
}

fn normalized_folder_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub async fn import_opml(state: &AppState, xml: &str) -> AppResult<OpmlImportResultDto> {
    let opml_feeds = parse_opml(xml)?;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();

    for feed in opml_feeds {
        let folder_id = match feed.folder.as_deref() {
            Some(name) => {
                let (display, normalized) =
                    match crate::validation::normalize_display_name(name, "Folder name") {
                        Ok(value) => value,
                        Err(_) => ("Imported folder".to_string(), "imported folder".to_string()),
                    };
                let connection = state.connection.lock().map_err(|_| {
                    AppError::Internal("The local database is temporarily unavailable.".to_string())
                })?;
                let profile_id = db::settings::get_active_profile_id(&connection)?;
                let folders = db::feeds::list_folders(&connection, &profile_id)?;
                let existing = folders
                    .iter()
                    .find(|item| normalized_folder_name(&item.name) == normalized);
                if let Some(existing) = existing {
                    Some(existing.id.clone())
                } else {
                    let new_id = uuid::Uuid::new_v4().to_string();
                    db::feeds::create_folder(
                        &connection,
                        &profile_id,
                        &new_id,
                        &display,
                        &normalized,
                    )?;
                    Some(new_id)
                }
            }
            None => None,
        };

        match subscribe(
            state,
            &feed.url,
            folder_id.as_deref(),
            feed.title.as_deref(),
        )
        .await
        {
            Ok(result) => {
                if result.duplicate {
                    skipped += 1;
                } else {
                    imported += 1;
                }
            }
            Err(error) => {
                skipped += 1;
                errors.push(format!("{}: {}", feed.url, error.user_message()));
            }
        }
    }

    Ok(OpmlImportResultDto {
        imported,
        skipped,
        errors,
    })
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn export_opml(state: &AppState) -> AppResult<String> {
    let connection = state.connection.lock().map_err(|_| {
        AppError::Internal("The local database is temporarily unavailable.".to_string())
    })?;
    let profile_id = db::settings::get_active_profile_id(&connection)?;
    let feeds = db::feeds::list_feeds(&connection, &profile_id)?;
    let folders = db::feeds::list_folders(&connection, &profile_id)?;

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<opml version=\"2.0\">\n<head>\n<title>Bergag subscriptions</title>\n</head>\n<body>\n",
    );
    for folder in folders {
        let folder_feeds: Vec<&FeedDto> = feeds
            .iter()
            .filter(|feed| feed.folder_id.as_deref() == Some(folder.id.as_str()))
            .collect();
        if folder_feeds.is_empty() {
            continue;
        }
        xml.push_str(&format!(
            "  <outline text=\"{}\" title=\"{}\">\n",
            escape_xml(&folder.name),
            escape_xml(&folder.name)
        ));
        for feed in folder_feeds {
            xml.push_str(&format!(
                "    <outline type=\"rss\" text=\"{}\" title=\"{}\" xmlUrl=\"{}\" htmlUrl=\"{}\"/>\n",
                escape_xml(&feed.title),
                escape_xml(&feed.title),
                escape_xml(&feed.feed_url),
                escape_xml(feed.site_url.as_deref().unwrap_or("")),
            ));
        }
        xml.push_str("  </outline>\n");
    }
    for feed in feeds.iter().filter(|feed| feed.folder_id.is_none()) {
        xml.push_str(&format!(
            "  <outline type=\"rss\" text=\"{}\" title=\"{}\" xmlUrl=\"{}\" htmlUrl=\"{}\"/>\n",
            escape_xml(&feed.title),
            escape_xml(&feed.title),
            escape_xml(&feed.feed_url),
            escape_xml(feed.site_url.as_deref().unwrap_or("")),
        ));
    }
    xml.push_str("</body>\n</opml>\n");
    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_opml_with_folders_and_duplicates() {
        let xml = r#"<?xml version="1.0"?><opml version="2.0"><body>
          <outline text="Tech" title="Tech">
            <outline text="A" title="A" xmlUrl="https://example.com/a.xml"/>
            <outline text="B" xmlUrl="https://example.com/b.xml"/>
          </outline>
          <outline text="A again" xmlUrl="https://example.com/a.xml"/>
        </body></opml>"#;
        let feeds = parse_opml(xml).unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].folder.as_deref(), Some("Tech"));
        assert_eq!(feeds[1].folder.as_deref(), Some("Tech"));
        assert_eq!(feeds[0].title.as_deref(), Some("A"));
    }

    #[test]
    fn rejects_invalid_opml() {
        assert!(parse_opml("<opml><body>").is_err());
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(escape_xml("A & B <C>"), "A &amp; B &lt;C&gt;");
    }
}
