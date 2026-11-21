use ammonia::{Builder, UrlRelative};
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use url::Url;

const MAX_RAW_HTML_CHARS: usize = 2_000_000;
const MAX_TEXT_CHARS: usize = 200_000;
pub const MAX_IMAGE_URLS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedContent {
    pub text: String,
    pub safe_html: String,
    pub word_count: i64,
    pub image_urls: Vec<String>,
}

fn limited_chars(value: &str, maximum: usize) -> String {
    if value.chars().count() <= maximum {
        return value.to_string();
    }
    value.chars().take(maximum).collect()
}

pub fn plain_text(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    let mut words = Vec::new();
    for text in fragment.root_element().text() {
        words.extend(text.split_whitespace().map(str::to_string));
    }
    limited_chars(&words.join(" "), MAX_TEXT_CHARS)
}

fn builder(base: Option<&Url>) -> Builder<'static> {
    let mut sanitizer = Builder::empty();
    sanitizer
        .add_tags(&[
            "a",
            "article",
            "b",
            "blockquote",
            "br",
            "code",
            "col",
            "colgroup",
            "dd",
            "del",
            "div",
            "dl",
            "dt",
            "em",
            "figcaption",
            "figure",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "hr",
            "i",
            "img",
            "li",
            "ol",
            "p",
            "pre",
            "q",
            "s",
            "section",
            "small",
            "span",
            "strong",
            "sub",
            "sup",
            "table",
            "tbody",
            "td",
            "th",
            "thead",
            "time",
            "tr",
            "u",
            "ul",
        ])
        .add_clean_content_tags(&[
            "script", "style", "iframe", "form", "svg", "math", "noscript", "nav", "aside",
            "footer", "header", "object", "embed", "video", "audio", "canvas",
        ])
        .url_schemes(HashSet::from(["http", "https", "mailto"]))
        .link_rel(Some("noopener noreferrer"))
        .strip_comments(true);

    let mut attributes = HashMap::new();
    attributes.insert("a", HashSet::from(["href", "title"]));
    attributes.insert(
        "img",
        HashSet::from(["src", "alt", "title", "width", "height"]),
    );
    attributes.insert("blockquote", HashSet::from(["cite"]));
    attributes.insert("q", HashSet::from(["cite"]));
    attributes.insert("time", HashSet::from(["datetime"]));
    sanitizer.tag_attributes(attributes);

    match base {
        Some(url) => {
            sanitizer.url_relative(UrlRelative::RewriteWithBase(url.clone()));
        }
        None => {
            sanitizer.url_relative(UrlRelative::Deny);
        }
    }
    sanitizer
}

pub fn extract_image_urls(safe_html: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse("img[src]") else {
        return Vec::new();
    };
    let fragment = Html::parse_fragment(safe_html);
    let mut seen = HashSet::new();
    let mut urls = Vec::new();
    for element in fragment.select(&selector) {
        let Some(src) = element.value().attr("src") else {
            continue;
        };
        let trimmed = src.trim();
        if trimmed.is_empty() || trimmed.starts_with("data:") || !seen.insert(trimmed.to_string()) {
            continue;
        }
        urls.push(trimmed.to_string());
        if urls.len() >= MAX_IMAGE_URLS {
            break;
        }
    }
    urls
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Replaces cached image `src` values with data URLs. Any image that could not
/// be cached has its remote `src` removed so the webview never makes a network request.
pub fn rewrite_image_sources(safe_html: &str, data_urls: &HashMap<String, String>) -> String {
    let pattern = regex::Regex::new(r#"src="([^"]*)""#).expect("static image source pattern");
    pattern
        .replace_all(safe_html, |captures: &regex::Captures<'_>| {
            let original = captures.get(1).map(|value| value.as_str()).unwrap_or("");
            match data_urls.get(&html_unescape(original)) {
                Some(data_url) => format!("src=\"{data_url}\""),
                None => String::new(),
            }
        })
        .to_string()
}

pub fn sanitize(raw_html: &str, base: Option<&Url>) -> SanitizedContent {
    let source = limited_chars(raw_html, MAX_RAW_HTML_CHARS);
    let safe_html = builder(base).clean(&source).to_string();
    let image_urls = extract_image_urls(&safe_html);
    let text = plain_text(&safe_html);
    let word_count = text.split_whitespace().count() as i64;
    SanitizedContent {
        text,
        safe_html,
        word_count,
        image_urls,
    }
}

pub fn plain_text_source(summary: &str) -> SanitizedContent {
    let escaped = summary
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let safe_html = format!("<p>{escaped}</p>");
    let word_count = summary.split_whitespace().count() as i64;
    SanitizedContent {
        text: summary.to_string(),
        safe_html,
        word_count,
        image_urls: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_scripts_attributes_and_relative_urls() {
        let base = Url::parse("https://example.com/blog/post").unwrap();
        let result = sanitize(
            r#"<p onclick="bad()">Hello <b>world</b><script>alert(1)</script></p><a href="/next">Next</a><img src="../a.png" onerror="bad()">"#,
            Some(&base),
        );
        assert!(result.safe_html.contains("<b>world</b>"));
        assert!(result
            .safe_html
            .contains("href=\"https://example.com/next\""));
        assert!(result
            .safe_html
            .contains("src=\"https://example.com/a.png\""));
        assert!(!result.safe_html.contains("onclick"));
        assert!(!result.safe_html.contains("onerror"));
        assert!(!result.safe_html.contains("script"));
        assert_eq!(result.text, "Hello world Next");
    }

    #[test]
    fn rewrites_cached_images_and_removes_remote_images() {
        let html = r#"<p>Text</p><img src="https://example.com/a.png" alt="A"><img src="https://example.com/b.png">"#;
        let mut cached = HashMap::new();
        cached.insert(
            "https://example.com/a.png".to_string(),
            "data:image/png;base64,AAAA".to_string(),
        );
        let result = rewrite_image_sources(html, &cached);
        assert!(result.contains("src=\"data:image/png;base64,AAAA\""));
        assert!(!result.contains("https://example.com"));
        assert!(result.contains("alt=\"A\""));
    }
}
