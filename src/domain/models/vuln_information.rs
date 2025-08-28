use std::fmt;

use chrono::{DateTime, Utc};
use modql::field::Fields;
use serde::{Deserialize, Serialize};

use crate::domain::models::page_utils::PageFilter;

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow, Deserialize,
)]
pub struct VulnInformation {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cve: String,
    pub disclosure: String,
    pub solutions: String,
    pub reference_links: Vec<String>,
    pub tags: Vec<String>,
    pub github_search: Vec<String>,
    pub source: String,
    pub source_name: String,
    pub reasons: Vec<String>,
    pub pushed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields, Serialize)]
pub struct CreateVulnInformation {
    pub key: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cve: String,
    pub disclosure: String,
    pub solutions: String,
    pub reference_links: Vec<String>,
    pub tags: Vec<String>,
    pub github_search: Vec<String>,
    pub source: String,
    pub source_name: String,
    pub reasons: Vec<String>,
    pub pushed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListVulnInformationRequest {
    pub page_filter: PageFilter,
    pub search_params: SearchParams,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SearchParams {
    pub cve: Option<String>,
    pub title: Option<String>,
    pub pushed: Option<bool>,
    pub source_name: Option<String>,
}

impl SearchParams {
    pub fn new() -> Self {
        SearchParams::default()
    }

    pub fn with_cve(mut self, cve: Option<String>) -> Self {
        self.cve = cve;
        self
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

impl ListVulnInformationRequest {
    pub fn new(page_filter: PageFilter, search_params: SearchParams) -> Self {
        ListVulnInformationRequest {
            page_filter,
            search_params,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListVulnInformationResponseData {
    pub total: i64,
    pub data: Vec<VulnInformation>,
}

impl ListVulnInformationResponseData {
    pub fn new(total: i64, data: Vec<VulnInformation>) -> Self {
        ListVulnInformationResponseData { total, data }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetVulnInformationRequest {
    pub id: i64,
}

impl GetVulnInformationRequest {
    pub fn new(id: i64) -> Self {
        GetVulnInformationRequest { id }
    }
}
