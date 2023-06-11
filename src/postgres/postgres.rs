use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures_util::StreamExt;
use sqlx::{postgres::PgNotification, PgPool};

use crate::listener::Listener;

use super::builder::PgListenerBuilder;

/// postgres implementation of [`Listener`](super::event::Listener)
pub struct PgListener<P> {
    listener: sqlx::postgres::PgListener,
    connection_method: ConnectionMethod,
    _payload: PhantomData<P>,
}

enum ConnectionMethod {
    Config(PgListenerConfig),
    Pool(PgPool),
}

impl<P> Listener for PgListener<P>
where
    P: Send + Sync + From<PgNotification>,
{
    type Data = P;
    type S = BoxStream<'static, Self::Data>;

    fn into_stream(self) -> Self::S {
        self.listener
            .into_stream()
            .filter_map(|result: Result<PgNotification, sqlx::Error>| async move {
                result.ok().map(Into::into)
            })
            .boxed()
    }
}

impl<P> PgListener<P> {
    pub fn builder() -> PgListenerBuilder<P> {
        PgListenerBuilder::new()
    }

    /// Consume [`PgListenerConfig`](PgListenerConfig), then connect and listen to the specify channel
    pub async fn connect(config: PgListenerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut listener = sqlx::postgres::PgListener::connect(&config.url).await?;

        let channels: Vec<&str> = config.channels.iter().map(|x| x.as_str()).collect();

        listener.listen_all(channels).await?;

        let connection_method = ConnectionMethod::Config(config);

        Ok(PgListener {
            listener,
            connection_method,
            _payload: Default::default(),
        })
    }

    /// Listen to postgres directly from `PgPool`
    pub async fn from_pool(
        pool: &sqlx::postgres::PgPool,
        channels: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut listener = sqlx::postgres::PgListener::connect_with(pool).await?;
        let channels: Vec<&str> = channels.iter().map(|x| x.as_str()).collect();

        listener.listen_all(channels).await?;

        let connection_method = ConnectionMethod::Pool(pool.clone());

        Ok(PgListener {
            listener,
            connection_method,
            _payload: Default::default(),
        })
    }
}

pub struct PgListenerConfig {
    pub url: String,
    pub channels: Vec<String>,
}
