use super::content::{self, SanitizedContent};
use crate::error::{AppError, AppResult};
use crate::feeds::network;
use scraper::{Html, Selector};
use url::Url;

pub const MAX_ARTICLE_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct CachedImage {
    pub url: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

const CONTENT_SELECTORS: &[&str] = &[
    "article",
    "main",
    "[role=\"main\"]",
    ".entry-content",
    ".post-content",
    ".article-content",
    "#content",
    "body",
];

fn content_fragment(document: &Html) -> String {
    for selector in CONTENT_SELECTORS {
        let Ok(parsed) = Selector::parse(selector) else {
            continue;
        };
        if let Some(element) = document.select(&parsed).next() {
            let html = element.html();
            if content::plain_text(&html).chars().count() >= 80 {
                return html;
            }
        }
    }
    document.root_element().html()
}

fn sniff_mime(bytes: &[u8], header: Option<&str>) -> Option<String> {
    let header = header
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(
        header.as_str(),
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/avif"
    ) {
        return Some(header);
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Some("image/png".to_string());
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg".to_string());
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif".to_string());
    }
    if bytes.len() > 11 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp".to_string());
    }
    None
}

async fn fetch_image(url: &str) -> AppResult<CachedImage> {
    let parsed = network::parse_public_url(url)?;
    let result = network::fetch_bytes(&parsed, None, None, MAX_IMAGE_BYTES).await?;
    if !result.status.is_success() {
        return Err(AppError::Validation(
            "An article image could not be fetched.".to_string(),
        ));
    }
    let mime = sniff_mime(&result.bytes, result.content_type.as_deref()).ok_or_else(|| {
        AppError::Validation("Only common raster article images can be cached.".to_string())
    })?;
    Ok(CachedImage {
        url: parsed.to_string(),
        mime,
        bytes: result.bytes,
    })
}

pub fn extract_from_html(html: &str, base: &Url) -> AppResult<SanitizedContent> {
    let document = Html::parse_document(html);
    let fragment = content_fragment(&document);
    Ok(content::sanitize(&fragment, Some(base)))
}

pub async fn cache_article_images(content: &SanitizedContent) -> Vec<CachedImage> {
    let mut images = Vec::new();
    for url in &content.image_urls {
        match fetch_image(url).await {
            Ok(image) => images.push(image),
            Err(_) => continue,
        }
        if images.len() >= content::MAX_IMAGE_URLS {
            break;
        }
    }
    images
}

pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    use base64::Engine;
    format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_article_body_instead_of_page_chrome() {
        let html = r#"<html><body><nav>Home About Contact Navigation Links Here</nav><article><h1>Real title</h1><p>This is the article body with enough words to be selected by the extractor instead of the page navigation chrome.</p></article><footer>Footer links and legal text</footer></body></html>"#;
        let base = Url::parse("https://example.com/post").unwrap();
        let result = extract_from_html(html, &base).unwrap();
        assert!(result.text.contains("Real title"));
        assert!(!result.text.contains("Footer links"));
    }

    #[test]
    fn accepts_only_raster_image_types() {
        assert_eq!(
            sniff_mime(&[0x89, b'P', b'N', b'G'], None).as_deref(),
            Some("image/png")
        );
        assert_eq!(sniff_mime(b"<svg/>", Some("image/svg+xml")), None);
    }
}
