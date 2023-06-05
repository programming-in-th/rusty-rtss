use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures_util::StreamExt;
use sqlx::postgres::PgNotification;

use super::listener::Listener;

/// postgres implementation of [`Listener`](super::event::Listener)
pub struct PgListener<P> {
    listener: sqlx::postgres::PgListener,
    _payload: PhantomData<P>,
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
    /// Consume [`PgListenerConfig`](PgListenerConfig), then connect and listen to the specify channel
    pub async fn connect(config: PgListenerConfig<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut con = sqlx::postgres::PgListener::connect(config.url).await?;

        con.listen_all(config.channels).await?;

        Ok(PgListener {
            listener: con,
            _payload: Default::default(),
        })
    }

    pub async fn from_pool<'a, T>(pool: &sqlx::postgres::PgPool, channels: Vec<&'a str>) -> Result<Self, Box<dyn std::error::Error>>
    {
        let mut con = sqlx::postgres::PgListener::connect_with(pool).await?;

        con.listen_all(channels).await?;

        Ok(PgListener {
            listener: con,
            _payload: Default::default(),
        })
    }
}

pub struct PgListenerConfig<'a> {
    pub url: &'a str,
    pub channels: Vec<&'a str>,
}
