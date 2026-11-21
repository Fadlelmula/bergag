use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedFolderDto {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub feed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedDto {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub site_url: Option<String>,
    pub feed_url: String,
    pub feed_type: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub status: String,
    pub last_refreshed_at: Option<String>,
    pub last_success_at: Option<String>,
    pub next_refresh_at: String,
    pub refresh_interval_minutes: i64,
    pub consecutive_failures: i64,
    pub last_error: Option<String>,
    pub article_count: i64,
    pub health: String,
    pub created_at: String,
    pub updated_at: String,
    pub category: Option<String>,
    pub trust_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedItemDto {
    pub id: String,
    pub feed_id: String,
    pub guid: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefreshLogDto {
    pub id: String,
    pub feed_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub status: String,
    pub http_status: Option<i64>,
    pub item_count: i64,
    pub new_item_count: i64,
    pub duration_ms: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredFeedDto {
    pub title: String,
    pub url: String,
    pub feed_type: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedPreviewDto {
    pub title: String,
    pub site_url: Option<String>,
    pub feed_type: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub item_count: i64,
    pub items: Vec<FeedItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscribeResultDto {
    pub feed: FeedDto,
    pub preview: FeedPreviewDto,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefreshOutcomeDto {
    pub feed_id: String,
    pub title: String,
    pub status: String,
    pub http_status: Option<i64>,
    pub item_count: i64,
    pub new_item_count: i64,
    pub duration_ms: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpmlImportResultDto {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

pub fn health_for_failures(failures: i64) -> &'static str {
    match failures {
        0 => "healthy",
        1..=2 => "degraded",
        _ => "failing",
    }
}
