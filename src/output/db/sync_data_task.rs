use error_stack::Result;
use sea_query::Value;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
    errors::Error,
    output::db::base::{Dao, dao_create, dao_first, dao_update, dao_update_field},
};

pub struct SyncDataTaskDao;

impl Dao for SyncDataTaskDao {
    const TABLE: &'static str = "sync_task";
}

impl SyncDataTaskDao {
    pub async fn first(tx: &mut Transaction<'_, Postgres>) -> Result<Option<SyncDataTask>, Error> {
        let task = dao_first::<Self, _>(tx).await?;
        Ok(task)
    }

    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateSyncDataTaskRequest,
    ) -> Result<i64, Error> {
        let task: Option<SyncDataTask> = dao_first::<Self, _>(tx).await?;
        if let Some(t) = task {
            dao_update::<Self, _>(tx, t.id, req).await?;
            return Ok(t.id);
        }
        let ret = dao_create::<Self, _>(tx, req).await?;
        Ok(ret)
    }

    pub async fn update_job(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
        job_id: String,
    ) -> Result<u64, Error> {
        let row = dao_update_field::<Self>(tx, id, "job_id", Value::String(Some(Box::new(job_id))))
            .await?;
        Ok(row)
    }
}
