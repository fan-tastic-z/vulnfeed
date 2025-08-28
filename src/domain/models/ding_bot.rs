use modql::field::Fields;
use nutype::nutype;
use sea_query::Value;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct DingBotConfig {
    pub id: i64,
    pub access_token: String,
    pub secret_token: String,
    pub status: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct CreateDingBotRequest {
    pub access_token: DingAccessToken,
    pub secret_token: DingSecretToken,
    pub status: bool,
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct DingAccessToken(String);

impl From<DingAccessToken> for Value {
    fn from(value: DingAccessToken) -> Self {
        Value::String(Some(Box::new(value.into_inner())))
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct DingSecretToken(String);

impl From<DingSecretToken> for Value {
    fn from(value: DingSecretToken) -> Self {
        Value::String(Some(Box::new(value.into_inner())))
    }
}
