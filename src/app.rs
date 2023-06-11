use crate::listener::Connector;

use super::{listener::Listener, publisher::Publisher};

use futures_util::{Stream, StreamExt};
use std::sync::Arc;

struct Inner<C, P> {
    connector: C,
    publisher: P,
}

/// Wrapper around [Inner](Inner)
/// All of the logic should be perform in [Inner](Inner)
pub struct App<C, P> {
    inner: Arc<Inner<C, P>>,
}

impl<C, P> App<C, P> {
    pub async fn new<T>(connector: C, publisher: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
        C: Connector + 'static,
        <C as Connector>::Listener: Listener<Data = T> + 'static,
    {
        let inner = Arc::new(Inner {
            connector,
            publisher,
        });

        let app = App { inner };

        Ok(app)
    }

    pub async fn add_subscriber<S>(&self, subscriber: S) -> Result<(), Box<dyn std::error::Error>>
    where
        P: Publisher<Subscriber = S> + 'static,
    {
        let inner = Arc::clone(&self.inner);

        if let Err(e) = inner.add_subscriber(subscriber).await {
            log::warn!("Unable to add subscriber: {e:?}");
        };

        tokio::task::yield_now().await;

        Ok(())
    }

    pub fn add_stream<S, T>(&self, stream: S) -> tokio::task::JoinHandle<()>
    where
        S: Stream<Item = T> + Send + 'static,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        Arc::clone(&self.inner).add_stream(stream)
    }

    pub async fn handle_connection<T>(&self) -> Result<(), tokio::task::JoinError>
    where
        C: Connector + 'static,
        <C as Connector>::Listener: Listener<Data = T>,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        Arc::clone(&self.inner).handle_connection().await
    }
}

impl<C, P> Clone for App<C, P> {
    fn clone(&self) -> Self {
        App {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<C, P> Inner<C, P> {
    pub async fn add_subscriber<S>(
        self: Arc<Self>,
        subscriber: S,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        P: Publisher<Subscriber = S> + 'static,
    {
        self.publisher.add_subscriber(subscriber);

        Ok(())
    }

    pub fn add_stream<S, T>(self: Arc<Self>, stream: S) -> tokio::task::JoinHandle<()>
    where
        S: Stream<Item = T> + Send + 'static,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        tokio::spawn(async move {
            stream
                .for_each_concurrent(10, move |payload| Arc::clone(&self).handle_payload(payload))
                .await;

            log::info!("Stream end");
        })
    }

    async fn handle_payload<T>(self: Arc<Self>, payload: T)
    where
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        self.publisher.publish(payload).await
    }

    /// Future become ready after connector give up on connection
    async fn handle_connection<T>(self: Arc<Self>) -> Result<(), tokio::task::JoinError>
    where
        C: Connector + 'static,
        <C as Connector>::Listener: Listener<Data = T>,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        tokio::spawn(async move {
            while let Some(listener) = self.connector.connect().await {
                let stream_handle = Arc::clone(&self).add_stream(listener.into_stream());

                stream_handle.await?
            }
            log::info!("Connector give up");
            Ok(())
        })
        .await
        .flatten()
    }
}
