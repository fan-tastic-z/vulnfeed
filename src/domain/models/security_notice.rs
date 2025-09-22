use std::fmt;

use chrono::{DateTime, Utc};
use modql::field::Fields;
use serde::{Deserialize, Serialize};

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
