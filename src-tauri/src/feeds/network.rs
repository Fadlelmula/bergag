use crate::error::{AppError, AppResult};
use crate::security::redaction::redact;
use reqwest::header::{ETAG, LAST_MODIFIED};
use reqwest::redirect::Policy;
use reqwest::StatusCode;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use url::Url;

pub const REQUEST_TIMEOUT_SECONDS: u64 = 20;
pub const MAX_REDIRECT_HOPS: u32 = 5;
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
    // url::Host already recognizes bracketed IP literals, which
    // host_str().parse::<IpAddr>() would miss because of the brackets.
    match url.host() {
        Some(url::Host::Ipv4(address)) => is_public_ip(IpAddr::V4(address)),
        Some(url::Host::Ipv6(address)) => is_public_ip(IpAddr::V6(address)),
        Some(url::Host::Domain(host)) => {
            let lowered = host.trim_end_matches('.').to_ascii_lowercase();
            !(lowered.is_empty()
                || lowered == "localhost"
                || lowered.ends_with(".localhost")
                || lowered.ends_with(".local")
                || lowered.ends_with(".internal")
                || lowered.ends_with(".home.arpa"))
        }
        None => false,
    }
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

fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .user_agent(concat!("Bergag/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .connect_timeout(Duration::from_secs(10))
        // Automatic redirect following is disabled: a hostname target is only
        // string-checked by is_public_url, not DNS-verified. fetch_bytes
        // follows redirects manually and re-validates every hop through
        // client_for, which resolves and pins a verified public IP.
        .redirect(Policy::none())
}

fn is_redirect_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}

/// Resolves a redirect `Location` against the current URL and applies the
/// same string-level checks as the original feed URL. Hostname targets are
/// additionally resolved and pinned to a public IP by client_for on the next
/// hop, so a public-looking name that resolves to a private address is
/// rejected before any request is sent to it.
fn resolve_redirect(current: &Url, location: &str) -> AppResult<Url> {
    let trimmed = location.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "The feed server sent an empty redirect.".to_string(),
        ));
    }
    let mut next = current.join(trimmed).map_err(|_| {
        AppError::Validation("The feed server sent an invalid redirect.".to_string())
    })?;
    next.set_fragment(None);
    if !is_public_url(&next) {
        return Err(AppError::Validation(
            "The feed server redirected to a blocked address.".to_string(),
        ));
    }
    Ok(next)
}

/// Picks the first verified public address from a resolver result. Split
/// from client_for so the rejection of hostnames that resolve only to
/// private addresses is testable with injected addresses, not live DNS.
fn pick_public_address(addrs: impl IntoIterator<Item = SocketAddr>) -> AppResult<SocketAddr> {
    addrs
        .into_iter()
        .find(|candidate| is_public_ip(candidate.ip()))
        .ok_or_else(|| {
            AppError::Validation("The feed host does not resolve to a public address.".to_string())
        })
}

pub async fn client_for(url: &Url) -> AppResult<reqwest::Client> {
    if url.host().is_none() {
        return Err(AppError::Validation("URL has no host.".to_string()));
    }
    let mut builder = client_builder();
    // IP literals need no DNS resolution or pinning; only domain names do.
    if let Some(url::Host::Domain(host)) = url.host() {
        let port = url.port_or_known_default().unwrap_or(443);
        let addresses = tokio::net::lookup_host((host, port))
            .await
            .map_err(|error| {
                tracing::warn!(error = %redact(&error.to_string()), "feed host lookup failed");
                AppError::Internal("The feed host could not be resolved.".to_string())
            })?;
        let address = pick_public_address(addresses)?;
        builder = builder.resolve(host, address);
    }
    builder.build().map_err(|error| {
        tracing::error!(error = %redact(&error.to_string()), "could not build feed HTTP client");
        AppError::Internal("The local HTTP client could not be initialized.".to_string())
    })
}

/// Conditional validators belong to the original resource, so they are
/// sent with the first request only and dropped after any redirect, even a
/// same-host one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConditionalHeaders {
    etag: Option<String>,
    last_modified: Option<String>,
}

/// One completed request, decoupled from reqwest so the redirect loop can
/// be driven by a scripted mock in tests.
struct RawResponse {
    status: StatusCode,
    location: Option<String>,
    etag: Option<String>,
    last_modified: Option<String>,
    content_type: Option<String>,
    body: Vec<u8>,
}

enum Hop {
    Follow(Url),
    Done,
}

/// Decides what a completed response means for the redirect loop: follow,
/// stop, or fail. Pure, so the loop's policy is testable without a server.
fn next_hop(
    current: &Url,
    status: StatusCode,
    location: Option<&str>,
    hops: u32,
) -> AppResult<Hop> {
    if !is_redirect_status(status) {
        return Ok(Hop::Done);
    }
    // A redirect status without a Location header is a final response.
    let Some(location) = location else {
        return Ok(Hop::Done);
    };
    if hops >= MAX_REDIRECT_HOPS {
        return Err(AppError::Validation(format!(
            "The feed redirected too many times (maximum {MAX_REDIRECT_HOPS})."
        )));
    }
    Ok(Hop::Follow(resolve_redirect(current, location)?))
}

/// Sends one request to one validated host. Every hop goes through
/// client_for, so every host, including redirect targets, is resolved and
/// pinned to a verified public IP before a request is sent to it.
async fn send_once(
    url: Url,
    conditional: Option<ConditionalHeaders>,
    max_bytes: usize,
) -> AppResult<RawResponse> {
    let client = client_for(&url).await?;
    let mut request = client.get(url);
    if let Some(conditional) = conditional {
        if let Some(etag) = conditional.etag {
            request = request.header(ETAG, etag);
        }
        if let Some(last_modified) = conditional.last_modified {
            request = request.header(LAST_MODIFIED, last_modified);
        }
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
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response
        .bytes()
        .await
        .map_err(|error| {
            tracing::warn!(error = %redact(&error.to_string()), "feed response read failed");
            AppError::Internal("The feed response could not be read.".to_string())
        })?
        .to_vec();

    Ok(RawResponse {
        status,
        location,
        etag,
        last_modified,
        content_type,
        body,
    })
}

/// Drives the redirect loop over any single-request fetcher. Hop counting,
/// stopping and the post-read size check live here; per-host validation and
/// DNS pinning live in the fetcher (send_once).
async fn fetch_with<F, Fut>(
    start: &Url,
    conditional: Option<ConditionalHeaders>,
    max_bytes: usize,
    mut fetch: F,
) -> AppResult<FetchResult>
where
    F: FnMut(Url, Option<ConditionalHeaders>) -> Fut,
    Fut: Future<Output = AppResult<RawResponse>>,
{
    let mut current = start.clone();
    let mut conditional = conditional;
    let mut hops = 0u32;
    loop {
        let response = fetch(current.clone(), conditional.take()).await?;
        if response.body.len() > max_bytes {
            return Err(AppError::Validation(format!(
                "Response is too large (maximum {} MB).",
                max_bytes / (1024 * 1024)
            )));
        }
        match next_hop(
            &current,
            response.status,
            response.location.as_deref(),
            hops,
        )? {
            Hop::Follow(next) => {
                hops += 1;
                current = next;
            }
            Hop::Done => {
                return Ok(FetchResult {
                    status: response.status,
                    etag: response.etag,
                    last_modified: response.last_modified,
                    content_type: response.content_type,
                    bytes: response.body,
                });
            }
        }
    }
}

pub async fn fetch_bytes(
    url: &Url,
    etag: Option<&str>,
    last_modified: Option<&str>,
    max_bytes: usize,
) -> AppResult<FetchResult> {
    let conditional = if etag.is_some() || last_modified.is_some() {
        Some(ConditionalHeaders {
            etag: etag.map(str::to_string),
            last_modified: last_modified.map(str::to_string),
        })
    } else {
        None
    };
    fetch_with(url, conditional, max_bytes, |current, conditional| {
        send_once(current, conditional, max_bytes)
    })
    .await
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
        assert!(parse_public_url("http://[::1]/feed").is_err());
        assert!(parse_public_url("http://[fd00::1]/feed").is_err());
        assert!(parse_public_url("http://[fe80::1]/feed").is_err());
        assert!(parse_public_url("http://10.0.0.5/feed").is_err());
        assert!(parse_public_url("file:///etc/passwd").is_err());
        assert!(parse_public_url("https://user:pass@example.com/feed").is_err());
    }

    #[test]
    fn resolve_redirect_joins_relative_and_absolute_locations() {
        let current = Url::parse("https://example.com/feeds/index.xml").unwrap();
        let relative = resolve_redirect(&current, "/feed.xml").unwrap();
        assert_eq!(relative.as_str(), "https://example.com/feed.xml");
        let cross_host = resolve_redirect(&current, "https://cdn.example.com/feed.xml").unwrap();
        assert_eq!(cross_host.host_str(), Some("cdn.example.com"));
    }

    #[test]
    fn resolve_redirect_rejects_blocked_targets() {
        let current = Url::parse("https://example.com/feed.xml").unwrap();
        // Literal private, loopback and link-local IPs.
        assert!(resolve_redirect(&current, "http://192.168.1.1/feed").is_err());
        assert!(resolve_redirect(&current, "http://127.0.0.1/feed").is_err());
        assert!(resolve_redirect(&current, "http://169.254.169.254/latest/meta-data").is_err());
        assert!(resolve_redirect(&current, "http://[::1]/feed").is_err());
        assert!(resolve_redirect(&current, "http://[fd00::1]/feed").is_err());
        assert!(resolve_redirect(&current, "http://[fe80::1]/feed").is_err());
        // Localhost-style names.
        assert!(resolve_redirect(&current, "http://localhost/feed").is_err());
        assert!(resolve_redirect(&current, "http://router.local/feed").is_err());
        // Non-http schemes and credential-bearing URLs.
        assert!(resolve_redirect(&current, "file:///etc/passwd").is_err());
        assert!(resolve_redirect(&current, "ftp://example.com/feed").is_err());
        assert!(resolve_redirect(&current, "https://user:pass@example.com/feed").is_err());
        // Empty and unparsable locations.
        assert!(resolve_redirect(&current, "   ").is_err());
        assert!(resolve_redirect(&current, "http://exa mple.com/feed").is_err());
    }

    #[test]
    fn redirect_statuses_are_detected() {
        assert!(is_redirect_status(StatusCode::MOVED_PERMANENTLY));
        assert!(is_redirect_status(StatusCode::FOUND));
        assert!(is_redirect_status(StatusCode::SEE_OTHER));
        assert!(is_redirect_status(StatusCode::TEMPORARY_REDIRECT));
        assert!(is_redirect_status(StatusCode::PERMANENT_REDIRECT));
        assert!(!is_redirect_status(StatusCode::OK));
        assert!(!is_redirect_status(StatusCode::NOT_MODIFIED));
    }

    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
            .block_on(future)
    }

    type Step = (StatusCode, Option<String>, Vec<u8>);

    fn ok(body: &str) -> Step {
        (StatusCode::OK, None, body.as_bytes().to_vec())
    }

    fn redirect(location: &str) -> Step {
        (StatusCode::FOUND, Some(location.to_string()), Vec::new())
    }

    /// Drives fetch_with with a scripted mock fetcher instead of a network,
    /// recording exactly which URLs were requested and with which
    /// conditional headers.
    fn run_script(
        start: &str,
        conditional: Option<ConditionalHeaders>,
        max_bytes: usize,
        steps: Vec<Step>,
    ) -> (
        AppResult<FetchResult>,
        Vec<(Url, Option<ConditionalHeaders>)>,
    ) {
        let script = Rc::new(RefCell::new(VecDeque::from(steps)));
        let record = Rc::new(RefCell::new(Vec::new()));
        let script2 = Rc::clone(&script);
        let record2 = Rc::clone(&record);
        let result = block_on(fetch_with(
            &Url::parse(start).unwrap(),
            conditional,
            max_bytes,
            move |url: Url, conditional: Option<ConditionalHeaders>| {
                let script = Rc::clone(&script2);
                let record = Rc::clone(&record2);
                async move {
                    record.borrow_mut().push((url, conditional));
                    let (status, location, body) = script
                        .borrow_mut()
                        .pop_front()
                        .expect("the redirect loop made an unscripted request");
                    Ok(RawResponse {
                        status,
                        location,
                        etag: None,
                        last_modified: None,
                        content_type: None,
                        body,
                    })
                }
            },
        ));
        let calls = record.borrow().clone();
        (result, calls)
    }

    fn expect_success(result: AppResult<FetchResult>) -> FetchResult {
        match result {
            Ok(result) => result,
            Err(error) => panic!("fetch should succeed: {}", error.user_message()),
        }
    }

    fn expect_failure(result: AppResult<FetchResult>) -> AppError {
        match result {
            Ok(_) => panic!("fetch should have failed"),
            Err(error) => error,
        }
    }

    #[test]
    fn follows_redirect_chain_and_requests_each_hop_host() {
        let conditional = Some(ConditionalHeaders {
            etag: Some("\"v1\"".to_string()),
            last_modified: Some("Mon, 01 Jan 2024".to_string()),
        });
        let (result, calls) = run_script(
            "https://example.com/feed",
            conditional.clone(),
            1024,
            vec![
                redirect("https://cdn.example.com/feed.xml"),
                ok("feed-body"),
            ],
        );
        let result = expect_success(result);
        assert_eq!(result.status, StatusCode::OK);
        assert_eq!(result.bytes, b"feed-body");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.as_str(), "https://example.com/feed");
        assert_eq!(calls[0].1, conditional);
        assert_eq!(calls[1].0.as_str(), "https://cdn.example.com/feed.xml");
    }

    #[test]
    fn drops_conditional_headers_after_any_redirect_including_same_host() {
        // Validators describe the original resource; forwarding them to a
        // redirect target would corrupt conditional caching.
        let (result, calls) = run_script(
            "https://example.com/feed",
            Some(ConditionalHeaders {
                etag: Some("\"v1\"".to_string()),
                last_modified: None,
            }),
            1024,
            vec![redirect("/moved"), ok("moved-body")],
        );
        expect_success(result);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].0.as_str(), "https://example.com/moved");
        assert_eq!(calls[1].1, None);
    }

    #[test]
    fn rejects_redirect_to_private_address_before_requesting_it() {
        let (result, calls) = run_script(
            "https://example.com/feed",
            None,
            1024,
            vec![redirect("http://192.168.1.1/admin")],
        );
        let error = expect_failure(result);
        assert!(matches!(error, AppError::Validation(_)));
        assert!(error.user_message().contains("blocked"));
        // The private target was never requested.
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn rejects_link_local_metadata_redirect_before_requesting_it() {
        let (result, calls) = run_script(
            "https://example.com/feed",
            None,
            1024,
            vec![redirect("http://169.254.169.254/latest/meta-data")],
        );
        expect_failure(result);
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn gives_up_after_max_redirect_hops() {
        let steps = vec![redirect("https://example.com/loop"); (MAX_REDIRECT_HOPS + 1) as usize];
        let (result, calls) = run_script("https://example.com/start", None, 1024, steps);
        let error = expect_failure(result);
        assert!(error.user_message().contains("too many"));
        assert_eq!(calls.len(), (MAX_REDIRECT_HOPS + 1) as usize);
    }

    #[test]
    fn treats_redirect_status_without_location_as_final() {
        let (result, calls) = run_script(
            "https://example.com/feed",
            None,
            1024,
            vec![(StatusCode::MOVED_PERMANENTLY, None, b"gone".to_vec())],
        );
        let result = expect_success(result);
        assert_eq!(result.status, StatusCode::MOVED_PERMANENTLY);
        assert_eq!(result.bytes, b"gone");
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn rejects_oversized_bodies_after_reading() {
        let big = vec![b'x'; 2048];
        let (result, _) = run_script(
            "https://example.com/feed",
            None,
            1024,
            vec![(StatusCode::OK, None, big)],
        );
        expect_failure(result);
    }

    #[test]
    fn pick_public_address_rejects_all_private_resolutions() {
        let private: Vec<SocketAddr> = vec![
            "192.168.1.10:443".parse().unwrap(),
            "169.254.169.254:443".parse().unwrap(),
            "[fd00::1]:443".parse().unwrap(),
        ];
        let error = match pick_public_address(private) {
            Ok(address) => panic!("private-only DNS answers must fail, got {address}"),
            Err(error) => error,
        };
        assert!(error.user_message().contains("public address"));
    }

    #[test]
    fn pick_public_address_selects_first_public_answer() {
        let mixed: Vec<SocketAddr> = vec![
            "10.0.0.4:443".parse().unwrap(),
            "93.184.216.34:443".parse().unwrap(),
        ];
        let picked = match pick_public_address(mixed) {
            Ok(address) => address,
            Err(error) => panic!("a public answer exists: {}", error.user_message()),
        };
        assert_eq!(picked, "93.184.216.34:443".parse().unwrap());
    }
}
