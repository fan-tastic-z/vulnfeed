use sea_query::Value;
use sqlx::{Postgres, Transaction};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, SecuritNotice},
    output::db::base::{Dao, dao_create, dao_fetch_by_column, dao_fetch_by_id, dao_update_field},
};

pub struct SecurityNoticeDao;

impl Dao for SecurityNoticeDao {
    const TABLE: &'static str = "security_notice";
}

impl SecurityNoticeDao {
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateSecurityNotice,
    ) -> AppResult<i64> {
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }

    pub async fn fetch_by_key(
        tx: &mut Transaction<'_, Postgres>,
        key: &str,
    ) -> AppResult<Option<SecuritNotice>> {
        dao_fetch_by_column::<Self, SecuritNotice>(tx, "key", key).await
    }

    pub async fn fetch_by_id(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> AppResult<Option<SecuritNotice>> {
        dao_fetch_by_id::<Self, SecuritNotice>(tx, id).await
    }

    pub async fn update_pushed(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
        status: bool,
    ) -> AppResult<u64> {
        let row = dao_update_field::<Self>(tx, id, "pushed", Value::Bool(Some(status))).await?;
        Ok(row)
    }
}
