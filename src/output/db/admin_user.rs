use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::admin_user::{AdminUser, AdminUsername, CreateAdminUserRequest},
    errors::Error,
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
    ) -> Result<i64, Error> {
        dao_upsert::<Self, _>(tx, req, "name", &["password"]).await
    }

    pub async fn fetch_by_name(
        tx: &mut Transaction<'_, Postgres>,
        name: &AdminUsername,
    ) -> Result<Option<AdminUser>, Error> {
        dao_fetch_by_column::<Self, AdminUser>(tx, "name", name.as_ref()).await
    }
}
