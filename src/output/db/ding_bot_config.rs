use sqlx::{Postgres, Transaction};

use crate::{
    AppResult,
    domain::models::ding_bot::{CreateDingBotRequest, DingBotConfig},
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
    ) -> AppResult<i64> {
        let ding_bot_config: Option<DingBotConfig> = dao_first::<Self, _>(tx).await?;
        if let Some(config) = ding_bot_config {
            dao_update::<Self, _>(tx, config.id, req).await?;
            return Ok(config.id);
        }
        let ret = dao_create::<Self, _>(tx, req).await?;
        Ok(ret)
    }

    pub async fn first(tx: &mut Transaction<'_, Postgres>) -> AppResult<Option<DingBotConfig>> {
        let ding_bot_config = dao_first::<Self, _>(tx).await?;
        Ok(ding_bot_config)
    }
}
