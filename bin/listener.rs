use crate::config::Config;
use sqlx::postgres::PgPool;

use super::Payload;
use super::Result;
use rusty_rtss::postgres::PgListener;

pub async fn get_pool_from_config(config: &Config) -> Result<PgPool> {
    PgPool::connect(&config.postgres.uri)
        .await
        .map_err(Into::into)
}

pub async fn get_listener_from_pool(pool: &PgPool, config: &Config) -> Result<PgListener<Payload>> {
    let channels = &config.postgres.listen_channels;

    let channels = channels.iter().map(|x| x.as_str()).collect();

    PgListener::from_pool(pool, channels)
        .await
        .map_err(|_| "unable to get listener from pool".into())
}
