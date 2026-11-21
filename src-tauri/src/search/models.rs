use crate::articles::models::ArticleListItemDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResultDto {
    pub item: ArticleListItemDto,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchPresetDto {
    pub id: String,
    pub name: String,
    pub query: String,
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchStatsDto {
    pub indexed_documents: i64,
    pub presets: i64,
    pub story_groups: i64,
    pub story_items: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryGroupDto {
    pub id: String,
    pub title: String,
    pub item_count: i64,
    pub source_count: i64,
    pub latest_at: Option<String>,
    pub created_at: String,
    pub items: Vec<ArticleListItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryTimelineBucketDto {
    pub date: String,
    pub stories: Vec<StoryGroupDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelatedArticleDto {
    pub item: ArticleListItemDto,
    pub score: i64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryDetectionDto {
    pub groups_created: i64,
    pub items_grouped: i64,
}
