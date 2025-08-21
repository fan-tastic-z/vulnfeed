use std::fmt;

use chrono::{DateTime, Utc};
use modql::field::Fields;
use serde::Serialize;

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
