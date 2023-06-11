use std::marker::PhantomData;

use sqlx::PgPool;

use super::{PgListener, PgListenerConfig};

pub struct PgListenerBuilder<P> {
    url: Option<String>,
    pool: Option<PgPool>,
    listen_channels: Vec<String>,
    _payload: PhantomData<P>,
}

impl<P> PgListenerBuilder<P> {
    pub(super) fn new() -> Self {
        PgListenerBuilder {
            url: None,
            pool: None,
            listen_channels: Vec::with_capacity(1),
            _payload: Default::default(),
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);

        self
    }

    pub fn with_pool(mut self, pool: &PgPool) -> Self {
        self.pool = Some(pool.clone());

        self
    }

    pub fn add_channel(mut self, channel: String) -> Self {
        self.listen_channels.push(channel);

        self
    }

    pub fn add_channels(mut self, channels: Vec<String>) -> Self {
        self.listen_channels.reserve(channels.len());

        for channel in channels {
            self.listen_channels.push(channel);
        }

        self
    }

    pub async fn build(self) -> Result<PgListener<P>, Box<dyn std::error::Error>> {
        let url = self.url;
        let pool = self.pool;
        let listen_channels = self.listen_channels;

        match (url, pool) {
            (None, None) | (Some(..), Some(..)) => {
                Err("Either url or pool needed to be supplied".into())
            }
            (None, Some(pool)) => Self::build_with_pool(pool, listen_channels).await,
            (Some(url), None) => Self::build_with_url(url, listen_channels).await,
        }
    }

    async fn build_with_url(
        url: String,
        listen_channels: Vec<String>,
    ) -> Result<PgListener<P>, Box<dyn std::error::Error>> {
        let config = PgListenerConfig {
            channels: listen_channels,
            url,
        };

        PgListener::connect(config).await
    }

    async fn build_with_pool(
        pool: PgPool,
        listen_channels: Vec<String>,
    ) -> Result<PgListener<P>, Box<dyn std::error::Error>> {
        PgListener::from_pool(&pool, listen_channels).await
    }
}
