use std::fmt;

use chrono::{DateTime, Utc};
use modql::field::Fields;
use serde::{Deserialize, Serialize};

use crate::domain::models::page_utils::PageFilter;

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow, Deserialize,
)]
pub struct SecuritNotice {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub product_name: String,
    pub risk_level: String,
    pub source: String,
    pub source_name: String,
    pub is_zero_day: bool,
    pub publish_time: String,
    pub detail_link: String,
    pub pushed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields, Serialize)]
pub struct CreateSecurityNotice {
    pub key: String,
    pub title: String,
    pub product_name: String,
    pub risk_level: String,
    pub source: String,
    pub source_name: String,
    pub is_zero_day: bool,
    pub publish_time: String,
    pub detail_link: String,
    pub pushed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListSecNoticeRequest {
    pub page_filter: PageFilter,
    pub search_params: SearchParams,
}

impl ListSecNoticeRequest {
    pub fn new(page_filter: PageFilter, search_params: SearchParams) -> Self {
        ListSecNoticeRequest {
            page_filter,
            search_params,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SearchParams {
    pub title: Option<String>,
    pub pushed: Option<bool>,
    pub source_name: Option<String>,
}

impl SearchParams {
    pub fn new() -> Self {
        SearchParams::default()
    }

    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }
    pub fn with_pushed(mut self, pushed: Option<bool>) -> Self {
        self.pushed = pushed;
        self
    }
    pub fn with_source_name(mut self, source_name: Option<String>) -> Self {
        self.source_name = source_name;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListSecNoticeResponseData {
    pub total: i64,
    pub data: Vec<SecuritNotice>,
}

impl ListSecNoticeResponseData {
    pub fn new(total: i64, data: Vec<SecuritNotice>) -> Self {
        ListSecNoticeResponseData { total, data }
    }
}
