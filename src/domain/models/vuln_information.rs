use std::fmt;

use chrono::{DateTime, Utc};
use modql::field::Fields;
use serde::Serialize;

use crate::domain::models::page_utils::PageFilter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
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
    pub reasons: Vec<String>,
    pub pushed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
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
    pub search: Option<String>,
}

impl ListVulnInformationRequest {
    pub fn new(page_filter: PageFilter, search: Option<String>) -> Self {
        ListVulnInformationRequest {
            page_filter,
            search,
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
