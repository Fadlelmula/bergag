use super::models::{IntegrationPlatformDto, ResolvedIntegrationSource};
use crate::error::{AppError, AppResult};
use crate::feeds::network;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationKind {
    YouTube,
    Reddit,
    GitHub,
    HackerNews,
    ArXiv,
    Podcast,
    Medium,
    Substack,
    Mastodon,
    Newsletter,
    WebsiteFeed,
    Bluesky,
    WebsiteMonitor,
}

impl IntegrationKind {
    pub const ALL: &'static [Self] = &[
        Self::YouTube,
        Self::Reddit,
        Self::GitHub,
        Self::HackerNews,
        Self::ArXiv,
        Self::Podcast,
        Self::Medium,
        Self::Substack,
        Self::Mastodon,
        Self::Newsletter,
        Self::WebsiteFeed,
        Self::Bluesky,
        Self::WebsiteMonitor,
    ];

    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw {
            "youtube" => Ok(Self::YouTube),
            "reddit" => Ok(Self::Reddit),
            "github" => Ok(Self::GitHub),
            "hacker_news" => Ok(Self::HackerNews),
            "arxiv" => Ok(Self::ArXiv),
            "podcast" => Ok(Self::Podcast),
            "medium" => Ok(Self::Medium),
            "substack" => Ok(Self::Substack),
            "mastodon" => Ok(Self::Mastodon),
            "newsletter" => Ok(Self::Newsletter),
            "website_feed" => Ok(Self::WebsiteFeed),
            "bluesky" => Ok(Self::Bluesky),
            "website_monitor" => Ok(Self::WebsiteMonitor),
            _ => Err(AppError::Validation(
                "Unknown integration platform.".to_string(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::YouTube => "youtube",
            Self::Reddit => "reddit",
            Self::GitHub => "github",
            Self::HackerNews => "hacker_news",
            Self::ArXiv => "arxiv",
            Self::Podcast => "podcast",
            Self::Medium => "medium",
            Self::Substack => "substack",
            Self::Mastodon => "mastodon",
            Self::Newsletter => "newsletter",
            Self::WebsiteFeed => "website_feed",
            Self::Bluesky => "bluesky",
            Self::WebsiteMonitor => "website_monitor",
        }
    }

    pub fn available(self) -> bool {
        !matches!(self, Self::Bluesky | Self::WebsiteMonitor)
    }

    fn unavailable_reason(self) -> Option<&'static str> {
        match self {
            Self::Bluesky => Some("Requires an authenticated API or a public bridge; planned."),
            Self::WebsiteMonitor => {
                Some("Change snapshots and alerting are planned. Add a public feed instead.")
            }
            _ => None,
        }
    }

    pub fn platform(self) -> IntegrationPlatformDto {
        let (label, description, input_label, input_hint) = match self {
            Self::YouTube => (
                "YouTube",
                "Subscribe to a public channel's video feed.",
                "Channel ID",
                "A 24-character ID beginning with UC.",
            ),
            Self::Reddit => (
                "Reddit",
                "Subscribe to a public subreddit feed.",
                "Subreddit",
                "For example, rust or rustlang.",
            ),
            Self::GitHub => (
                "GitHub",
                "Subscribe to a public repository's release feed.",
                "Repository",
                "For example, owner/repository.",
            ),
            Self::HackerNews => (
                "Hacker News",
                "Subscribe to the official front-page RSS feed.",
                "Feed",
                "Enter frontpage.",
            ),
            Self::ArXiv => (
                "arXiv",
                "Subscribe to a public arXiv category feed.",
                "Category",
                "For example, cs.AI or math.OC.",
            ),
            Self::Podcast => (
                "Podcast",
                "Subscribe to a podcast's public RSS feed.",
                "Feed URL",
                "A public http or https RSS/Atom URL.",
            ),
            Self::Medium => (
                "Medium",
                "Subscribe to a public Medium profile or topic feed.",
                "Feed URL",
                "A public medium.com/feed/... URL.",
            ),
            Self::Substack => (
                "Substack",
                "Subscribe to a public Substack publication feed.",
                "Publication name",
                "For example, example for example.substack.com.",
            ),
            Self::Mastodon => (
                "Mastodon",
                "Subscribe to a public Mastodon account feed.",
                "Account URL",
                "For example, https://mastodon.social/@user.",
            ),
            Self::Newsletter => (
                "Newsletter",
                "Subscribe to a newsletter's public RSS archive.",
                "Feed URL",
                "Only newsletters with a public feed can be added.",
            ),
            Self::WebsiteFeed => (
                "Website feed",
                "Subscribe to a website's public RSS or Atom feed.",
                "Feed URL",
                "A public http or https RSS/Atom URL.",
            ),
            Self::Bluesky => (
                "Bluesky",
                "Planned: public profile and feed adapter.",
                "Handle",
                "Not available in this build.",
            ),
            Self::WebsiteMonitor => (
                "Website monitoring",
                "Planned: page snapshots and change events.",
                "Website URL",
                "Not available in this build.",
            ),
        };
        IntegrationPlatformDto {
            kind: self.as_str().to_string(),
            label: label.to_string(),
            description: description.to_string(),
            input_label: input_label.to_string(),
            input_hint: input_hint.to_string(),
            available: self.available(),
            unavailable_reason: self.unavailable_reason().map(str::to_string),
        }
    }
}

pub fn platforms() -> Vec<IntegrationPlatformDto> {
    IntegrationKind::ALL
        .iter()
        .map(|kind| kind.platform())
        .collect()
}

fn normalize_input(raw: &str, label: &str, maximum: usize) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{label} cannot be empty.")));
    }
    if trimmed.chars().count() > maximum {
        return Err(AppError::Validation(format!(
            "{label} must be {maximum} characters or fewer."
        )));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{label} cannot contain control characters."
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_chars(
    value: &str,
    label: &str,
    minimum: usize,
    maximum: usize,
    allowed: impl Fn(char) -> bool,
) -> AppResult<()> {
    let count = value.chars().count();
    if count < minimum || count > maximum {
        return Err(AppError::Validation(format!(
            "{label} must be between {minimum} and {maximum} characters."
        )));
    }
    if !value.chars().all(allowed) {
        return Err(AppError::Validation(format!(
            "{label} contains invalid characters."
        )));
    }
    Ok(())
}

fn public_feed_url(raw: &str) -> AppResult<String> {
    let url = network::parse_public_url(raw)?;
    Ok(network::normalized_url(&url))
}

fn resolve_youtube(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let id = normalize_input(raw, "YouTube channel ID", 24)?;
    if id.chars().count() != 24 || !id.starts_with("UC") {
        return Err(AppError::Validation(
            "Enter a 24-character YouTube channel ID beginning with UC.".to_string(),
        ));
    }
    validate_chars(&id[2..], "Channel ID", 22, 22, |character| {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    })?;
    Ok(ResolvedIntegrationSource {
        source_key: id.clone(),
        source_url: Some(format!("https://www.youtube.com/channel/{id}")),
        feed_url: format!("https://www.youtube.com/feeds/videos.xml?channel_id={id}"),
        suggested_title: None,
    })
}

fn resolve_reddit(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let name = normalize_input(raw, "Subreddit", 21)?.to_ascii_lowercase();
    validate_chars(&name, "Subreddit", 3, 21, |character| {
        character.is_ascii_alphanumeric() || character == '_'
    })?;
    if !name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric())
    {
        return Err(AppError::Validation(
            "Subreddit names must start with a letter or number.".to_string(),
        ));
    }
    Ok(ResolvedIntegrationSource {
        source_key: name.clone(),
        source_url: Some(format!("https://www.reddit.com/r/{name}/")),
        feed_url: format!("https://www.reddit.com/r/{name}/.rss"),
        suggested_title: None,
    })
}

fn resolve_github(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let value = normalize_input(raw, "Repository", 160)?;
    let (owner, repository) = value.split_once('/').ok_or_else(|| {
        AppError::Validation("Enter a GitHub repository as owner/repository.".to_string())
    })?;
    let owner = normalize_input(owner, "Repository owner", 39)?;
    let repository = normalize_input(repository, "Repository name", 100)?;
    let allowed = |character: char| {
        character.is_ascii_alphanumeric()
            || character == '-'
            || character == '_'
            || character == '.'
    };
    validate_chars(&owner, "Repository owner", 1, 39, allowed)?;
    validate_chars(&repository, "Repository name", 1, 100, allowed)?;
    let key = format!(
        "{}/{}",
        owner.to_ascii_lowercase(),
        repository.to_ascii_lowercase()
    );
    Ok(ResolvedIntegrationSource {
        source_key: key,
        source_url: Some(format!("https://github.com/{owner}/{repository}")),
        feed_url: format!("https://github.com/{owner}/{repository}/releases.atom"),
        suggested_title: None,
    })
}

fn resolve_hacker_news(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let value = normalize_input(raw, "Feed", 32)?;
    if !value.eq_ignore_ascii_case("frontpage") {
        return Err(AppError::Validation(
            "Hacker News supports the official frontpage feed.".to_string(),
        ));
    }
    Ok(ResolvedIntegrationSource {
        source_key: "frontpage".to_string(),
        source_url: Some("https://news.ycombinator.com/".to_string()),
        feed_url: "https://news.ycombinator.com/rss".to_string(),
        suggested_title: Some("Hacker News front page".to_string()),
    })
}

fn resolve_arxiv(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let category = normalize_input(raw, "arXiv category", 32)?;
    validate_chars(&category, "arXiv category", 2, 32, |character| {
        character.is_ascii_alphanumeric() || character == '-' || character == '.'
    })?;
    if category.starts_with('.')
        || category.starts_with('-')
        || category.ends_with('.')
        || category.ends_with('-')
    {
        return Err(AppError::Validation(
            "arXiv categories cannot start or end with a dot or hyphen.".to_string(),
        ));
    }
    Ok(ResolvedIntegrationSource {
        source_key: category.to_ascii_lowercase(),
        source_url: Some(format!("https://arxiv.org/list/{category}/recent")),
        feed_url: format!("https://export.arxiv.org/rss/{category}"),
        suggested_title: Some(format!("arXiv {category}")),
    })
}

fn resolve_medium(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let url = network::parse_public_url(raw)?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::Validation("Enter a public Medium feed URL.".to_string()))?
        .to_ascii_lowercase();
    if host != "medium.com" && !host.ends_with(".medium.com") {
        return Err(AppError::Validation(
            "Medium integrations require a medium.com feed URL.".to_string(),
        ));
    }
    if url.path() != "/feed" && !url.path().starts_with("/feed/") {
        return Err(AppError::Validation(
            "Medium feed URLs must contain /feed/ in the path.".to_string(),
        ));
    }
    let feed_url = network::normalized_url(&url);
    Ok(ResolvedIntegrationSource {
        source_key: feed_url.clone(),
        source_url: Some(format!("https://{host}")),
        feed_url,
        suggested_title: None,
    })
}

fn resolve_substack(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let name = normalize_input(raw, "Substack publication name", 40)?.to_ascii_lowercase();
    validate_chars(&name, "Substack publication name", 2, 40, |character| {
        character.is_ascii_alphanumeric() || character == '-'
    })?;
    if name.starts_with('-') || name.ends_with('-') {
        return Err(AppError::Validation(
            "Substack names cannot start or end with a hyphen.".to_string(),
        ));
    }
    Ok(ResolvedIntegrationSource {
        source_key: name.clone(),
        source_url: Some(format!("https://{name}.substack.com")),
        feed_url: format!("https://{name}.substack.com/feed"),
        suggested_title: Some(format!("{name} on Substack")),
    })
}

fn resolve_mastodon(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let url = network::parse_public_url(raw)?;
    if url.scheme() != "https" {
        return Err(AppError::Validation(
            "Mastodon account URLs must use https.".to_string(),
        ));
    }
    let segments = url
        .path_segments()
        .ok_or_else(|| AppError::Validation("Enter a Mastodon account URL.".to_string()))?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let (handle, path) = match segments.as_slice() {
        [first] if first.starts_with('@') => {
            let handle = &first[1..];
            (handle, format!("@{handle}"))
        }
        ["users", handle] => (*handle, format!("users/{handle}")),
        _ => {
            return Err(AppError::Validation(
                "Enter a Mastodon profile URL such as https://instance/@user.".to_string(),
            ))
        }
    };
    validate_chars(handle, "Mastodon handle", 1, 30, |character| {
        character.is_ascii_alphanumeric() || character == '_'
    })?;
    let origin = url.origin().ascii_serialization();
    let source_url = format!("{origin}/{path}");
    Ok(ResolvedIntegrationSource {
        source_key: source_url.to_ascii_lowercase(),
        source_url: Some(source_url.clone()),
        feed_url: format!("{source_url}.rss"),
        suggested_title: Some(format!("{handle} on Mastodon")),
    })
}

fn resolve_public_url(raw: &str) -> AppResult<ResolvedIntegrationSource> {
    let feed_url = public_feed_url(raw)?;
    Ok(ResolvedIntegrationSource {
        source_key: feed_url.clone(),
        source_url: None,
        feed_url,
        suggested_title: None,
    })
}

pub fn resolve(kind: IntegrationKind, raw: &str) -> AppResult<ResolvedIntegrationSource> {
    match kind {
        IntegrationKind::YouTube => resolve_youtube(raw),
        IntegrationKind::Reddit => resolve_reddit(raw),
        IntegrationKind::GitHub => resolve_github(raw),
        IntegrationKind::HackerNews => resolve_hacker_news(raw),
        IntegrationKind::ArXiv => resolve_arxiv(raw),
        IntegrationKind::Podcast | IntegrationKind::Newsletter | IntegrationKind::WebsiteFeed => {
            resolve_public_url(raw)
        }
        IntegrationKind::Medium => resolve_medium(raw),
        IntegrationKind::Substack => resolve_substack(raw),
        IntegrationKind::Mastodon => resolve_mastodon(raw),
        IntegrationKind::Bluesky => Err(AppError::Validation(
            "Bluesky integration is planned and is not available in this build.".to_string(),
        )),
        IntegrationKind::WebsiteMonitor => Err(AppError::Validation(
            "Website change monitoring is planned. Add a public website feed instead.".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_available_and_planned_platforms() {
        let all = platforms();
        assert_eq!(all.len(), IntegrationKind::ALL.len());
        assert!(all
            .iter()
            .any(|platform| platform.kind == "youtube" && platform.available));
        assert!(all
            .iter()
            .any(|platform| platform.kind == "bluesky" && !platform.available));
        assert!(all
            .iter()
            .any(|platform| platform.kind == "website_monitor" && !platform.available));
    }

    #[test]
    fn normalizes_public_identifier_adapters() {
        let cases = [
            (
                IntegrationKind::YouTube,
                "UCabcdefghijklmnopqrstuv",
                "https://www.youtube.com/feeds/videos.xml?channel_id=UCabcdefghijklmnopqrstuv",
            ),
            (
                IntegrationKind::Reddit,
                " RustLang ",
                "https://www.reddit.com/r/rustlang/.rss",
            ),
            (
                IntegrationKind::GitHub,
                "rust-lang/rust",
                "https://github.com/rust-lang/rust/releases.atom",
            ),
            (
                IntegrationKind::HackerNews,
                "frontpage",
                "https://news.ycombinator.com/rss",
            ),
            (
                IntegrationKind::ArXiv,
                "cs.AI",
                "https://export.arxiv.org/rss/cs.AI",
            ),
            (
                IntegrationKind::Substack,
                "Example-News",
                "https://example-news.substack.com/feed",
            ),
            (
                IntegrationKind::Mastodon,
                "https://mastodon.social/@alice",
                "https://mastodon.social/@alice.rss",
            ),
        ];
        for (kind, input, expected) in cases {
            let resolved = resolve(kind, input).unwrap();
            assert_eq!(resolved.feed_url, expected, "unexpected URL for {kind:?}");
        }
    }

    #[test]
    fn normalizes_public_url_adapters() {
        let url = "https://example.com/feed.xml";
        for kind in [
            IntegrationKind::Podcast,
            IntegrationKind::Newsletter,
            IntegrationKind::WebsiteFeed,
            IntegrationKind::Medium,
        ] {
            let input = if kind == IntegrationKind::Medium {
                "https://medium.com/feed/@writer"
            } else {
                url
            };
            let resolved = resolve(kind, input).unwrap();
            assert_eq!(resolved.feed_url, input);
        }
    }

    #[test]
    fn rejects_invalid_and_planned_sources() {
        assert!(resolve(IntegrationKind::YouTube, "short").is_err());
        assert!(resolve(IntegrationKind::Reddit, "bad/name").is_err());
        assert!(resolve(IntegrationKind::GitHub, "owner").is_err());
        assert!(resolve(IntegrationKind::HackerNews, "newest").is_err());
        assert!(resolve(IntegrationKind::Podcast, "http://127.0.0.1/feed").is_err());
        assert!(resolve(IntegrationKind::Medium, "https://example.com/feed").is_err());
        assert!(resolve(IntegrationKind::Mastodon, "http://mastodon.social/@alice").is_err());
        assert!(resolve(IntegrationKind::Bluesky, "alice.bsky.social").is_err());
        assert!(resolve(IntegrationKind::WebsiteMonitor, "https://example.com").is_err());
    }

    #[test]
    fn parses_all_platform_kinds() {
        for kind in IntegrationKind::ALL {
            assert_eq!(IntegrationKind::parse(kind.as_str()).unwrap(), *kind);
        }
        assert!(IntegrationKind::parse("unknown").is_err());
    }
}
