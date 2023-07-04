use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures_util::StreamExt;
use sqlx::postgres::PgNotification;

use crate::listener::Listener;

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
    pub fn new(listener: sqlx::postgres::PgListener) -> Self {
        PgListener {
            listener,
            _payload: PhantomData,
        }
    }
}
