use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::ding_bot::{CreateDingBotRequest, DingBotConfig},
    errors::Error,
    output::db::base::{Dao, dao_create, dao_first, dao_update},
};

pub struct DingBotConfigDao;

impl Dao for DingBotConfigDao {
    const TABLE: &'static str = "ding_bot_config";
}

impl DingBotConfigDao {
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateDingBotRequest,
    ) -> Result<i64, Error> {
        let ding_bot_config: Option<DingBotConfig> = dao_first::<Self, _>(tx).await?;
        if let Some(config) = ding_bot_config {
            dao_update::<Self, _>(tx, config.id, req).await?;
            return Ok(config.id);
        }
        let ret = dao_create::<Self, _>(tx, req).await?;
        Ok(ret)
    }

    pub async fn first(tx: &mut Transaction<'_, Postgres>) -> Result<Option<DingBotConfig>, Error> {
        let ding_bot_config = dao_first::<Self, _>(tx).await?;
        Ok(ding_bot_config)
    }
}
