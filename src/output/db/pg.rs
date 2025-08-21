use error_stack::{Result, ResultExt};
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{config::settings::Config, errors::Error};

#[derive(Debug, Clone)]
pub struct Pg {
    pub pool: Pool<Postgres>,
}

impl Pg {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let opts = PgConnectOptions::new()
            .host(&config.database.host)
            .port(config.database.port)
            .username(&config.database.username)
            .password(&config.database.password)
            .database(&config.database.database_name);
        let pool = PgPoolOptions::new()
            .connect_with(opts)
            .await
            .change_context_lazy(|| Error::Message("failed to connect to database".to_string()))?;
        Ok(Self { pool })
    }
}
