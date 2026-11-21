use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationPlatformDto {
    pub kind: String,
    pub label: String,
    pub description: String,
    pub input_label: String,
    pub input_hint: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationSourceDto {
    pub id: String,
    pub kind: String,
    pub source_key: String,
    pub display_name: String,
    pub source_url: Option<String>,
    pub feed_url: String,
    pub feed_id: Option<String>,
    pub category: Option<String>,
    pub trust_level: String,
    pub status: String,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationPreviewResultDto {
    pub platform: IntegrationPlatformDto,
    pub resolved_feed_url: String,
    pub preview: crate::feeds::models::FeedPreviewDto,
    pub duplicate_feed: bool,
    pub duplicate_source: bool,
    pub existing_source: Option<IntegrationSourceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIntegrationSource {
    pub source_key: String,
    pub source_url: Option<String>,
    pub feed_url: String,
    pub suggested_title: Option<String>,
}
