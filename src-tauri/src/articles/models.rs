use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArticleStatePatch {
    pub is_read: Option<bool>,
    pub is_saved: Option<bool>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_queued: Option<bool>,
}

impl ArticleStatePatch {
    pub fn is_empty(&self) -> bool {
        self.is_read.is_none()
            && self.is_saved.is_none()
            && self.is_favorite.is_none()
            && self.is_archived.is_none()
            && self.is_queued.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleStateDto {
    pub is_read: bool,
    pub is_saved: bool,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_queued: bool,
    pub queue_position: Option<i64>,
    pub read_count: i64,
    pub last_read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleListItemDto {
    pub id: String,
    pub feed_id: String,
    pub feed_title: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub discovered_at: String,
    pub word_count: i64,
    pub state: ArticleStateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleImageDto {
    pub url: String,
    pub mime: String,
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleNoteDto {
    pub id: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleHighlightDto {
    pub id: String,
    pub text: String,
    pub start_offset: i64,
    pub end_offset: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollectionDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub article_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleDetailDto {
    pub item: ArticleListItemDto,
    pub source: String,
    pub content_text: String,
    pub safe_html: String,
    pub images: Vec<ArticleImageDto>,
    pub notes: Vec<ArticleNoteDto>,
    pub highlights: Vec<ArticleHighlightDto>,
    pub collections: Vec<CollectionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntryDto {
    pub id: String,
    pub item_id: String,
    pub title: String,
    pub feed_title: String,
    pub event_type: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightEntryDto {
    pub id: String,
    pub item_id: String,
    pub article_title: String,
    pub feed_title: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArticleCacheStatsDto {
    pub articles: i64,
    pub article_bytes: i64,
    pub images: i64,
    pub image_bytes: i64,
}
