use chrono::{DateTime, Utc};
use modql::field::Fields;
use nutype::nutype;
use sea_query::Value;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct CreateAdminUserRequest {
    pub name: AdminUsername,
    pub password: AdminUserPassword,
}

impl CreateAdminUserRequest {
    pub fn new(name: AdminUsername, password: AdminUserPassword) -> Self {
        Self { name, password }
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_min = 4, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct AdminUsername(String);

impl From<AdminUsername> for Value {
    fn from(admin_username: AdminUsername) -> Self {
        Value::String(Some(Box::new(admin_username.into_inner())))
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 8, len_char_max = 128),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom
    )
)]
pub struct AdminUserPassword(String);

impl From<AdminUserPassword> for Value {
    fn from(admin_password: AdminUserPassword) -> Self {
        Value::String(Some(Box::new(admin_password.into_inner())))
    }
}
