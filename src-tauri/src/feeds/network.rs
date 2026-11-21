use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use reqwest::header::{ETAG, LAST_MODIFIED};
use reqwest::redirect::Policy;
use reqwest::StatusCode;
use std::net::IpAddr;
use std::time::Duration;
use url::Url;

pub const REQUEST_TIMEOUT_SECONDS: u64 = 20;
pub const MAX_FEED_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_DISCOVERY_BYTES: usize = 2 * 1024 * 1024;

pub struct FetchResult {
    pub status: StatusCode,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

fn is_public_ipv4(octets: [u8; 4]) -> bool {
    let [a, b, c, d] = octets;
    if a == 0 || a == 10 || a == 127 || a >= 224 {
        return false;
    }
    if a == 100 && (64..=127).contains(&b) {
        return false;
    }
    if a == 169 && b == 254 {
        return false;
    }
    if a == 172 && (16..=31).contains(&b) {
        return false;
    }
    if a == 192 && b == 168 {
        return false;
    }
    if a == 192 && b == 0 && c == 0 {
        return false;
    }
    if a == 192 && b == 0 && c == 2 {
        return false;
    }
    if a == 198 && (b == 18 || b == 19) {
        return false;
    }
    if a == 198 && b == 51 && c == 100 {
        return false;
    }
    if a == 203 && b == 0 && c == 113 {
        return false;
    }
    !(a == 255 && b == 255 && c == 255 && d == 255)
}

fn is_public_ipv6(octets: [u8; 16]) -> bool {
    let first = u16::from_be_bytes([octets[0], octets[1]]);
    if first == 0 || first == 1 {
        return false;
    }
    if first & 0xfe00 == 0xfc00 || first & 0xffc0 == 0xfe80 || first & 0xff00 == 0xff00 {
        return false;
    }
    if octets[..10].iter().all(|byte| *byte == 0) && octets[10] == 0xff && octets[11] == 0xff {
        return is_public_ipv4([octets[12], octets[13], octets[14], octets[15]]);
    }
    true
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => is_public_ipv4(address.octets()),
        IpAddr::V6(address) => is_public_ipv6(address.octets()),
    }
}

pub fn is_public_url(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    if let Ok(address) = host.parse::<IpAddr>() {
        return is_public_ip(address);
    }
    let lowered = host.trim_end_matches('.').to_ascii_lowercase();
    if lowered.is_empty()
        || lowered == "localhost"
        || lowered.ends_with(".localhost")
        || lowered.ends_with(".local")
        || lowered.ends_with(".internal")
        || lowered.ends_with(".home.arpa")
    {
        return false;
    }
    true
}

pub fn parse_public_url(raw: &str) -> AppResult<Url> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Enter a feed or website URL.".to_string(),
        ));
    }
    if trimmed.len() > 2048 {
        return Err(AppError::Validation(
            "URL is too long (maximum 2048 characters).".to_string(),
        ));
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&with_scheme)
        .map_err(|_| AppError::Validation("Enter a valid http or https URL.".to_string()))?;
    url.set_fragment(None);
    if !is_public_url(&url) {
        return Err(AppError::Validation(
            "Only public http and https addresses are supported.".to_string(),
        ));
    }
    Ok(url)
}

pub fn normalized_url(url: &Url) -> String {
    url.to_string()
}

fn redirect_policy() -> Policy {
    Policy::custom(|attempt| {
        if is_public_url(attempt.url()) {
            attempt.follow()
        } else {
            attempt.stop()
        }
    })
}

fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .user_agent(concat!("Bergag/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .connect_timeout(Duration::from_secs(10))
        .redirect(redirect_policy())
}

pub async fn client_for(url: &Url) -> AppResult<reqwest::Client> {
    let Some(host) = url.host_str() else {
        return Err(AppError::Validation("URL has no host.".to_string()));
    };
    let mut builder = client_builder();
    if host.parse::<IpAddr>().is_err() {
        let port = url.port_or_known_default().unwrap_or(443);
        let mut addresses = tokio::net::lookup_host((host, port))
            .await
            .map_err(|error| {
                tracing::warn!(error = %redact(&error.to_string()), "feed host lookup failed");
                AppError::Internal("The feed host could not be resolved.".to_string())
            })?;
        let address = addresses
            .find(|candidate| is_public_ip(candidate.ip()))
            .ok_or_else(|| {
                AppError::Validation(
                    "The feed host does not resolve to a public address.".to_string(),
                )
            })?;
        builder = builder.resolve(host, address);
    }
    builder.build().map_err(|error| {
        tracing::error!(error = %redact(&error.to_string()), "could not build feed HTTP client");
        AppError::Internal("The local HTTP client could not be initialized.".to_string())
    })
}

pub async fn fetch_bytes(
    url: &Url,
    etag: Option<&str>,
    last_modified: Option<&str>,
    max_bytes: usize,
) -> AppResult<FetchResult> {
    let client = client_for(url).await?;
    let mut request = client.get(url.clone());
    if let Some(etag) = etag {
        request = request.header(ETAG, etag);
    }
    if let Some(last_modified) = last_modified {
        request = request.header(LAST_MODIFIED, last_modified);
    }
    let response = request.send().await.map_err(|error| {
        tracing::warn!(error = %redact(&error.to_string()), "feed request failed");
        AppError::Internal("The feed server could not be reached.".to_string())
    })?;

    if let Some(length) = response.content_length() {
        if length as usize > max_bytes {
            return Err(AppError::Validation(format!(
                "Response is too large (maximum {} MB).",
                max_bytes / (1024 * 1024)
            )));
        }
    }

    let status = response.status();
    let response_etag = response
        .headers()
        .get(ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let response_last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let response_content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = response.bytes().await.map_err(|error| {
        tracing::warn!(error = %redact(&error.to_string()), "feed response read failed");
        AppError::Internal("The feed response could not be read.".to_string())
    })?;
    if bytes.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "Response is too large (maximum {} MB).",
            max_bytes / (1024 * 1024)
        )));
    }

    Ok(FetchResult {
        status,
        etag: response_etag,
        last_modified: response_last_modified,
        content_type: response_content_type,
        bytes: bytes.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_public_urls_and_adds_https() {
        let url = parse_public_url("example.com/feed.xml").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(normalized_url(&url), "https://example.com/feed.xml");
    }

    #[test]
    fn rejects_private_and_unsafe_urls() {
        assert!(parse_public_url("http://localhost/feed").is_err());
        assert!(parse_public_url("http://127.0.0.1/feed").is_err());
        assert!(parse_public_url("http://10.0.0.5/feed").is_err());
        assert!(parse_public_url("file:///etc/passwd").is_err());
        assert!(parse_public_url("https://user:pass@example.com/feed").is_err());
    }

    #[test]
    fn redirect_policy_rejects_private_targets() {
        let private = Url::parse("http://192.168.1.1/feed").unwrap();
        let public = Url::parse("https://example.com/feed").unwrap();
        assert!(!is_public_url(&private));
        assert!(is_public_url(&public));
    }
}
