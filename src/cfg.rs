use config::{Config as ConfigIO, Environment};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RmqConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub vhost: String,
    pub queue_env: String,
    pub port: u32,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub replay_duration: u64,
}

#[derive(Deserialize, Clone)]
pub struct AxumConfig {
    pub listen_host: String,
    pub listen_port: u32,
}

pub struct Config {
    pub app_config: AppConfig,
    pub rmq_config: RmqConfig,
    pub axum_config: AxumConfig,
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let rmq_config = ConfigIO::builder()
        .add_source(Environment::with_prefix("RABBITMQ").prefix_separator("_"))
        .build()?
        .try_deserialize()?;

    let app_config = ConfigIO::builder()
        .add_source(Environment::with_prefix("APP").prefix_separator("_"))
        .build()?
        .try_deserialize()?;

    let axum_config = ConfigIO::builder()
        .add_source(Environment::with_prefix("AXUM").prefix_separator("_"))
        .build()?
        .try_deserialize()?;

    Ok(Config {
        app_config,
        rmq_config,
        axum_config,
    })
}
