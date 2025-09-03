use sqlx::{Postgres, Transaction};

use crate::{
    AppResult,
    domain::models::admin_user::{AdminUser, AdminUsername, CreateAdminUserRequest},
    output::db::base::{Dao, dao_fetch_by_column, dao_upsert},
};

pub struct AdminUserDao;

impl Dao for AdminUserDao {
    const TABLE: &'static str = "admin_user";
}

impl AdminUserDao {
    pub async fn create_super_user(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateAdminUserRequest,
    ) -> AppResult<i64> {
        dao_upsert::<Self, _>(tx, req, "name", &["password"]).await
    }

    pub async fn fetch_by_name(
        tx: &mut Transaction<'_, Postgres>,
        name: &AdminUsername,
    ) -> AppResult<Option<AdminUser>> {
        dao_fetch_by_column::<Self, AdminUser>(tx, "name", name.as_ref()).await
    }
}
