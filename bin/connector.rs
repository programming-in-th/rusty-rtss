use crate::config::Config;
use rusty_rtss::postgres::PgConnector;
use sqlx::postgres::PgPool;

use super::Payload;
use super::Result;

pub async fn get_pool_from_config(config: &Config) -> Result<PgPool> {
    PgPool::connect(&config.postgres.uri)
        .await
        .map_err(Into::into)
}

pub async fn get_connector_from_pool(
    pool: &PgPool,
    config: &Config,
) -> Result<PgConnector<Payload>> {
    let channels = config.postgres.listen_channels.clone();

    PgConnector::builder()
        .with_pool(pool)
        .add_channels(channels)
        .build()
        .await
        .map_err(|_| "Unable to get listener from pool".into())
}
