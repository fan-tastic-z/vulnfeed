use sea_query::Value;
use sqlx::{Postgres, Transaction};

use crate::{
    AppResult,
    domain::models::{
        page_utils::PageFilter,
        security_notice::{CreateSecurityNotice, SearchParams, SecuritNotice},
    },
    output::db::base::{
        Dao, DaoQueryBuilder, dao_create, dao_fetch_by_column, dao_fetch_by_id, dao_update_field,
    },
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

    pub async fn filter_security_notices(
        tx: &mut Transaction<'_, Postgres>,
        page_filter: &PageFilter,
        search_params: &SearchParams,
    ) -> AppResult<Vec<SecuritNotice>> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        if let Some(title) = &search_params.title {
            query_builder = query_builder.and_where_like("title", title);
        }

        if let Some(source_name) = &search_params.source_name {
            query_builder = query_builder.and_where_like("source_name", source_name);
        }

        if let Some(pushed) = &search_params.pushed {
            query_builder = query_builder.and_where_bool("pushed", *pushed);
        }

        let page_no = *page_filter.page_no().as_ref();
        let page_size = *page_filter.page_size().as_ref();
        let offset = (page_no - 1) * page_size;
        query_builder
            .order_by_desc("updated_at")
            .limit_offset(page_size as i64, offset as i64)
            .fetch_all(tx)
            .await
    }

    pub async fn filter_security_notices_count(
        tx: &mut Transaction<'_, Postgres>,
        search_params: &SearchParams,
    ) -> AppResult<i64> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        if let Some(title) = &search_params.title {
            query_builder = query_builder.and_where_like("title", title);
        }

        if let Some(source_name) = &search_params.source_name {
            query_builder = query_builder.and_where_like("source_name", source_name);
        }

        if let Some(pushed) = &search_params.pushed {
            query_builder = query_builder.and_where_bool("pushed", *pushed);
        }

        query_builder.count(tx).await
    }
}
