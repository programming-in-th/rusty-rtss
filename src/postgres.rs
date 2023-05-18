use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures_util::StreamExt;
use sqlx::postgres::PgNotification;

use super::listener::Listener;

pub trait Identifiable {
    type Identifier;

    fn from(input: PgNotification) -> (Self::Identifier, Self);
}

/// postgres implementation of [`Listener`](super::event::Listener)
pub struct PgListener<I, P> {
    listener: sqlx::postgres::PgListener,
    _payload: PhantomData<P>,
    _id: PhantomData<I>,
}

impl<I, P> Listener for PgListener<I, P>
where
    P: Send + Sync + Identifiable<Identifier = I>,
    I: Send + Sync,
{
    type Identifier = I;
    type Payload = P;
    type S = BoxStream<'static, (Self::Identifier, Self::Payload)>;

    fn into_stream(self) -> Self::S {
        self.listener
            .into_stream()
            .filter_map(|result: Result<PgNotification, sqlx::Error>| async move {
                if let Ok(x) = result {
                    Some(<P as Identifiable>::from(x))
                } else {
                    None
                }
            })
            .boxed()
    }
}

impl<I, P> PgListener<I, P> {
    /// Consume [`PgListenerConfig`](PgListenerConfig), then connect and listen to the specify channel
    pub async fn connect(config: PgListenerConfig<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut con = sqlx::postgres::PgListener::connect(config.url).await?;

        con.listen_all(config.channels).await?;

        Ok(PgListener {
            listener: con,
            _payload: Default::default(),
            _id: Default::default(),
        })
    }
}

pub struct PgListenerConfig<'a> {
    pub url: &'a str,
    pub channels: Vec<&'a str>,
}
