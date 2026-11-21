use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoverySourceDto {
    pub id: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub title: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub trust_level: String,
    pub subscribed_feed_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryPreviewResultDto {
    pub preview: crate::feeds::models::FeedPreviewDto,
    pub source: Option<DiscoverySourceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_json: String,
    pub enabled: bool,
    pub is_smart_feed: bool,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleRunDto {
    pub id: String,
    pub rule_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub status: String,
    pub matched_count: i64,
    pub changed_count: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MatchMode {
    All,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "field",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RuleCondition {
    TitleContains { value: String },
    SummaryContains { value: String },
    ContentContains { value: String },
    FeedIs { feed_id: String },
    FeedTitleContains { value: String },
    CategoryIs { value: String },
    TrustAtLeast { value: String },
    HasLabel { label_id: String },
    IsRead { value: bool },
    IsSaved { value: bool },
    IsFavorite { value: bool },
    IsArchived { value: bool },
    IsQueued { value: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "action",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RuleAction {
    MarkRead { value: bool },
    SetSaved { value: bool },
    SetFavorite { value: bool },
    SetArchived { value: bool },
    SetQueued { value: bool },
    AddLabel { label_id: String },
    RemoveLabel { label_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuleDefinition {
    pub match_mode: MatchMode,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
}

#[derive(Debug, Clone)]
pub struct RuleCandidate {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub content_text: String,
    pub feed_id: String,
    pub feed_title: String,
    pub category: String,
    pub trust_level: String,
    pub is_read: bool,
    pub is_saved: bool,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_queued: bool,
    pub label_ids: Vec<String>,
}
