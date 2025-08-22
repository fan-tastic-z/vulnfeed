use chrono::{DateTime, Utc};
use modql::field::Fields;
use nutype::nutype;
use sea_query::Value;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct SyncDataTask {
    pub id: i64,
    pub name: String,
    pub interval_minutes: i32,
    pub status: bool,
    pub job_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct CreateSyncDataTaskRequest {
    pub name: SyncDataTaskName,
    pub interval_minutes: SyncDataTaskIntervalMinutes,
    pub status: bool,
    pub job_id: Option<String>,
}

impl CreateSyncDataTaskRequest {
    pub fn new(
        name: SyncDataTaskName,
        interval_minutes: SyncDataTaskIntervalMinutes,
        status: bool,
    ) -> Self {
        Self {
            name,
            interval_minutes,
            status,
            job_id: None,
        }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 4, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct SyncDataTaskName(String);

impl From<SyncDataTaskName> for Value {
    fn from(name: SyncDataTaskName) -> Self {
        Value::String(Some(Box::new(name.into_inner())))
    }
}

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 60),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct SyncDataTaskIntervalMinutes(u32);

impl From<SyncDataTaskIntervalMinutes> for Value {
    fn from(interval_minutes: SyncDataTaskIntervalMinutes) -> Self {
        Value::Int(Some(interval_minutes.into_inner() as i32))
    }
}
