use crate::error::{AppError, AppResult};
use crate::feeds::models::DiscoveredFeedDto;
use crate::security::redaction::redact;
use feed_rs::model::FeedType as ParsedFeedType;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone)]
pub struct ParsedItem {
    pub guid: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content_html: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub site_url: Option<String>,
    pub feed_type: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub items: Vec<ParsedItem>,
}

fn truncate(value: String, maximum: usize) -> String {
    if value.chars().count() <= maximum {
        return value;
    }
    value.chars().take(maximum).collect()
}

fn clean_text(value: &str) -> String {
    let fragment = Html::parse_fragment(value);
    let mut result = String::new();
    for text in fragment.root_element().text() {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !result.is_empty() {
            let first = trimmed.chars().next().unwrap_or(' ');
            if !matches!(first, '.' | '!' | '?' | ',' | ';' | ':' | ')' | ']' | '}') {
                result.push(' ');
            }
        }
        result.push_str(trimmed);
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn feed_type_name(feed_type: &ParsedFeedType) -> &'static str {
    match feed_type {
        ParsedFeedType::Atom => "atom",
        ParsedFeedType::JSON => "json",
        ParsedFeedType::RSS0 | ParsedFeedType::RSS1 | ParsedFeedType::RSS2 => "rss",
    }
}

fn first_link(feed: &feed_rs::model::Feed) -> Option<String> {
    feed.links
        .iter()
        .find(|link| link.rel.as_deref().unwrap_or("alternate") == "alternate")
        .or_else(|| feed.links.first())
        .map(|link| truncate(link.href.clone(), 2048))
}

fn item_link(entry: &feed_rs::model::Entry) -> Option<String> {
    entry
        .links
        .iter()
        .find(|link| link.rel.as_deref().unwrap_or("alternate") == "alternate")
        .or_else(|| entry.links.first())
        .map(|link| truncate(link.href.clone(), 2048))
}

pub fn parse_feed_bytes(bytes: &[u8], source_url: &str) -> AppResult<ParsedFeed> {
    let feed = feed_rs::parser::parse(bytes).map_err(|error| {
        tracing::warn!(error = %redact(&error.to_string()), "feed parse failed");
        AppError::Validation("This URL did not return a valid RSS, Atom, or JSON feed.".to_string())
    })?;

    let title = feed
        .title
        .as_ref()
        .map(|text| clean_text(&text.content))
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "Untitled feed".to_string());
    let description = feed
        .description
        .as_ref()
        .map(|text| clean_text(&text.content))
        .filter(|text| !text.is_empty())
        .map(|text| truncate(text, 1000));
    let language = feed
        .language
        .as_ref()
        .map(|value| truncate(value.clone(), 32));

    let items = feed
        .entries
        .iter()
        .map(|entry| {
            let url = item_link(entry);
            let raw_title = entry
                .title
                .as_ref()
                .map(|text| clean_text(&text.content))
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "Untitled item".to_string());
            let published_at = entry
                .published
                .or(entry.updated)
                .map(|timestamp| timestamp.to_rfc3339());
            let guid_seed = if !entry.id.is_empty() {
                entry.id.clone()
            } else if let Some(url) = &url {
                url.clone()
            } else {
                format!(
                    "{source_url}|{raw_title}|{}",
                    published_at.as_deref().unwrap_or("")
                )
            };
            let guid = if guid_seed.is_empty() || guid_seed.len() > 512 {
                hash(&guid_seed)
            } else {
                guid_seed
            };
            let author = entry
                .authors
                .first()
                .and_then(|person| person.name.as_ref())
                .map(|name| truncate(clean_text(name), 128))
                .filter(|name| !name.is_empty());
            let summary = entry
                .summary
                .as_ref()
                .map(|text| text.content.clone())
                .or_else(|| {
                    entry
                        .content
                        .as_ref()
                        .and_then(|content| content.body.clone())
                })
                .map(|value| clean_text(&value))
                .filter(|value| !value.is_empty())
                .map(|value| truncate(value, 2000));

            let content_html = entry
                .content
                .as_ref()
                .and_then(|content| content.body.clone())
                .map(|value| truncate(value, 2_000_000));

            ParsedItem {
                guid,
                title: truncate(raw_title, 512),
                url,
                author,
                summary,
                content_html,
                published_at,
            }
        })
        .collect();

    Ok(ParsedFeed {
        title: truncate(title, 256),
        site_url: first_link(&feed),
        feed_type: feed_type_name(&feed.feed_type).to_string(),
        description,
        language,
        items,
    })
}

fn feed_type_from_hint(hint: &str, href: &str) -> &'static str {
    let lowered = hint.to_ascii_lowercase();
    if lowered.contains("atom") || href.to_ascii_lowercase().ends_with(".atom") {
        "atom"
    } else if lowered.contains("json") || href.to_ascii_lowercase().ends_with(".json") {
        "json"
    } else {
        "rss"
    }
}

pub fn discover_feeds(html: &[u8], source_url: &str) -> AppResult<Vec<DiscoveredFeedDto>> {
    let document = Html::parse_document(std::str::from_utf8(html).map_err(|_| {
        AppError::Validation("The website response was not valid UTF-8 text.".to_string())
    })?);
    let selector = Selector::parse("link[rel][href]").map_err(|_| {
        AppError::Internal("Feed discovery selector could not be initialized.".to_string())
    })?;
    let base = Url::parse(source_url)
        .map_err(|_| AppError::Validation("The website URL is invalid.".to_string()))?;
    let mut discovered = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for element in document.select(&selector) {
        let value = element.value();
        let rel = value.attr("rel").unwrap_or_default().to_ascii_lowercase();
        if !rel.split_whitespace().any(|item| item == "alternate") {
            continue;
        }
        let href = value.attr("href").unwrap_or_default().trim();
        if href.is_empty() {
            continue;
        }
        let hint = value.attr("type").unwrap_or_default();
        let hint_lower = hint.to_ascii_lowercase();
        let looks_like_feed = hint_lower.contains("rss")
            || hint_lower.contains("atom")
            || hint_lower.contains("json")
            || href.to_ascii_lowercase().ends_with(".xml")
            || href.to_ascii_lowercase().ends_with(".rss")
            || href.to_ascii_lowercase().ends_with(".atom");
        if !looks_like_feed {
            continue;
        }
        let Ok(absolute) = base.join(href) else {
            continue;
        };
        if !crate::feeds::network::is_public_url(&absolute) || !seen.insert(absolute.to_string()) {
            continue;
        }
        let title = value
            .attr("title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| absolute.host_str().unwrap_or("Discovered feed"));
        discovered.push(DiscoveredFeedDto {
            title: truncate(title.to_string(), 256),
            url: absolute.to_string(),
            feed_type: feed_type_from_hint(hint, href).to_string(),
            source_url: source_url.to_string(),
        });
        if discovered.len() >= 20 {
            break;
        }
    }

    Ok(discovered)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<?xml version="1.0"?>
    <rss version="2.0"><channel>
      <title>Example &amp; News</title>
      <link>https://example.com</link>
      <description>Daily notes</description>
      <language>en-us</language>
      <item>
        <title>First item</title>
        <link>https://example.com/1</link>
        <guid>item-1</guid>
        <description><![CDATA[<p>Hello <b>world</b>.</p>]]></description>
        <pubDate>Mon, 06 Sep 2010 00:01:00 +0000</pubDate>
      </item>
    </channel></rss>"#;

    const ATOM: &str = r#"<?xml version="1.0"?>
    <feed xmlns="http://www.w3.org/2005/Atom">
      <title>Atom Journal</title>
      <link rel="alternate" href="https://atom.example.com"/>
      <entry>
        <title>Entry title</title>
        <id>urn:entry:2</id>
        <link href="https://atom.example.com/2"/>
        <summary>Plain summary</summary>
      </entry>
    </feed>"#;

    #[test]
    fn parses_rss_without_html_content() {
        let parsed = parse_feed_bytes(RSS.as_bytes(), "https://example.com/feed.xml").unwrap();
        assert_eq!(parsed.title, "Example & News");
        assert_eq!(parsed.feed_type, "rss");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].guid, "item-1");
        assert_eq!(parsed.items[0].summary.as_deref(), Some("Hello world."));
    }

    #[test]
    fn parses_atom_entries() {
        let parsed = parse_feed_bytes(ATOM.as_bytes(), "https://atom.example.com/feed").unwrap();
        assert_eq!(parsed.feed_type, "atom");
        assert_eq!(parsed.items[0].guid, "urn:entry:2");
    }

    #[test]
    fn rejects_html_as_feed() {
        assert!(parse_feed_bytes(
            b"<html><body>not a feed</body></html>",
            "https://example.com"
        )
        .is_err());
    }

    #[test]
    fn discovers_alternate_feed_links() {
        let html = br#"<html><head>
          <link rel="alternate" type="application/rss+xml" title="News" href="/feed.xml">
          <link rel="stylesheet" href="/style.css">
        </head></html>"#;
        let feeds = discover_feeds(html, "https://example.com/site").unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed.xml");
        assert_eq!(feeds[0].title, "News");
    }
}
