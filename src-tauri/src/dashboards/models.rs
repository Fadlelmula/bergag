use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub widget_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardWidgetDto {
    pub id: String,
    pub dashboard_id: String,
    pub kind: String,
    pub title: String,
    pub days: i64,
    pub item_limit: i64,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}
