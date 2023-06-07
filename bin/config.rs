use super::Result;

pub struct Config {
    pub postgres: PostgresConfig,
    pub axum: AxumConfig,
}

pub struct PostgresConfig {
    pub uri: String,
    pub listen_channels: Vec<String>,
}

pub struct AxumConfig {
    pub port: u32,
    pub host: String,
}

pub fn load_config() -> Result<Config> {
    Ok(Config {
        postgres: load_postgres_config()?,
        axum: load_axum_config()?,
    })
}

fn load_postgres_config() -> Result<PostgresConfig> {
    let uri = env_string("POSTGRES_URI")?;
    let listen_channels = env_string("POSTGRES_LISTEN_CHANNELS")?;
    let listen_channels = listen_channels
        .split(',')
        .map(ToString::to_string)
        .collect();

    Ok(PostgresConfig {
        uri,
        listen_channels,
    })
}

fn load_axum_config() -> Result<AxumConfig> {
    let host = env_string("RTSS_HOST")?;
    let port = env_u32("RTSS_PORT")?;

    Ok(AxumConfig { host, port })
}

fn env_string(key: impl AsRef<str>) -> Result<String> {
    let key = key.as_ref();

    match std::env::var(key) {
        Ok(x) => Ok(x),
        Err(..) => Err(format!("Environment variable `{key}` is not set").into()),
    }
}

fn env_u32(key: impl AsRef<str>) -> Result<u32> {
    let key = key.as_ref();

    let raw = match std::env::var(key) {
        Ok(x) => x,
        Err(..) => return Err(format!("Environment variable `{key}` is not set").into()),
    };

    match raw.parse() {
        Ok(x) => Ok(x),
        Err(..) => Err(format!("Unable to parse environment variable `{key}`").into()),
    }
}
