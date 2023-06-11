use super::{listener::Listener, publisher::Publisher};

use futures_util::StreamExt;
use std::sync::Arc;

struct Inner<P> {
    publisher: P,
}

pub struct App<P> {
    inner: Arc<Inner<P>>,
}

impl<P> App<P> {
    pub fn new<L, T>(listener: L, publisher: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        L: Listener<Data = T> + 'static,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        let inner = Arc::new(Inner { publisher });

        let cloned_inner = Arc::clone(&inner);

        let app = App { inner };

        let _handle = tokio::spawn(cloned_inner.add_listener(listener));

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

    pub async fn add_listener<L, T>(
        &self,
        listener: L,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        L: Listener<Data = T> + 'static,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        let inner = Arc::clone(&self.inner);

        inner.add_listener(listener).await
    }
}

impl<P> Clone for App<P> {
    fn clone(&self) -> Self {
        App {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<P> Inner<P> {
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

    pub async fn add_listener<L, T>(
        self: Arc<Self>,
        listener: L,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        L: Listener<Data = T> + 'static,
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        let stream = listener.into_stream();

        tokio::spawn(async move {
            stream
                .for_each_concurrent(10, move |payload| Arc::clone(&self).handle_payload(payload))
                .await;
    
            log::info!("Stream end");
        });
        
        Ok(())
    }

    async fn handle_payload<T>(self: Arc<Self>, payload: T)
    where
        P: Publisher<PublishData = T> + 'static,
        T: Send + Sync + 'static,
    {
        let inner = Arc::clone(&self);

        inner.publisher.publish(payload).await
    }
}
