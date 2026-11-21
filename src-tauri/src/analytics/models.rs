use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityDayDto {
    pub date: String,
    pub opened: i64,
    pub read: i64,
    pub unread: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadingStatsDto {
    pub days: i64,
    pub activity: Vec<ActivityDayDto>,
    pub total_opened: i64,
    pub total_read: i64,
    pub highlights: i64,
    pub notes: i64,
    pub streak_days: i64,
    pub last_read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibraryStatsDto {
    pub articles: i64,
    pub feeds: i64,
    pub folders: i64,
    pub collections: i64,
    pub labels: i64,
    pub highlights: i64,
    pub notes: i64,
    pub stories: i64,
    pub presets: i64,
    pub words: i64,
    pub oldest_article_at: Option<String>,
    pub newest_article_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageStatsDto {
    pub database_bytes: i64,
    pub cached_image_bytes: i64,
    pub cached_images: i64,
    pub extracted_articles: i64,
    pub text_characters: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedHealthDto {
    pub id: String,
    pub title: String,
    pub status: String,
    pub health: String,
    pub consecutive_failures: i64,
    pub failures_30d: i64,
    pub last_refreshed_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
    pub items: i64,
    pub unread: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamedCountDto {
    pub id: String,
    pub name: String,
    pub count: i64,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingHighlightDto {
    pub item_id: String,
    pub article_title: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefingDto {
    pub period: String,
    pub since: String,
    pub generated_at: String,
    pub new_articles: i64,
    pub read_events: i64,
    pub unread_total: i64,
    pub highlights_saved: i64,
    pub notes_added: i64,
    pub top_feeds: Vec<NamedCountDto>,
    pub recent_highlights: Vec<BriefingHighlightDto>,
}
