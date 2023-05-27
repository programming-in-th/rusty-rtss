use crate::sse::Identifiable;

use super::{listener::Listener, publisher::Publisher};

use futures_util::StreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct App<P> {
    _handle: JoinHandle<()>,
    publisher: Arc<P>,
}

impl<P> App<P> {
    pub fn new<L, I, T>(listener: L, publisher: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        L: Listener<Data = T> + 'static,
        P: Publisher<PublishData = T> + 'static,
        I: Send + Sync + 'static,
        T: Send + Sync + 'static + Identifiable<Identifier = I>,
    {
        let publisher = Arc::new(publisher);

        let cloned_publisher = Arc::clone(&publisher);
        let handle = tokio::spawn(async move {
            let cloned_publisher = cloned_publisher;
            let stream = listener.into_stream();

            stream
                .for_each_concurrent(10, move |payload| {
                    let cloned_publisher = Arc::clone(&cloned_publisher);

                    async move {
                        let cloned_publisher = cloned_publisher;

                        cloned_publisher.publish(payload).await;
                    }
                })
                .await
        });

        Ok(App {
            _handle: handle,
            publisher: publisher,
        })
    }

    pub async fn add_subscriber<S>(&self, subscriber: S) -> Result<(), Box<dyn std::error::Error>>
    where
        P: Publisher<Subscriber = S> + 'static,
    {
        self.publisher.add_subscriber(subscriber);

        tokio::task::yield_now().await;

        Ok(())
    }
}
