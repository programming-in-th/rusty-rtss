use super::event::{Listener, Publisher};

use futures_util::StreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct App<P> {
    _handle: JoinHandle<()>,
    publisher: Arc<P>,
}

impl<P> App<P> {
    pub fn new<L, I, W, T>(listener: L, publisher: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        L: Listener<Payload = T, Identifier = I> + 'static,
        P: Publisher<Payload = T, Identifier = I, Writer = W> + 'static,
        I: Send + Sync + 'static,
        W: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let publisher = Arc::new(publisher);

        let cloned_publisher = Arc::clone(&publisher);
        let handle = tokio::spawn(async move {
            let cloned_publisher = cloned_publisher;
            let stream = listener.into_stream();

            stream
                .for_each_concurrent(10, move |(id, payload)| {
                    let cloned_publisher = Arc::clone(&cloned_publisher);

                    async move {
                        let cloned_publisher = cloned_publisher;

                        cloned_publisher.publish(&id, payload).await;
                    }
                })
                .await
        });

        Ok(App {
            _handle: handle,
            publisher: publisher,
        })
    }

    /// Push add subscriber event to message queue.
    pub async fn add_subscriber<I, W, T>(
        &self,
        id: I,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        P: Publisher<Payload = T, Identifier = I, Writer = W> + 'static,
        I: Send + Sync + 'static,
        W: Send + Sync + 'static,
    {
        self.publisher.add_subscriber(id, writer);

        tokio::task::yield_now().await;

        Ok(())
    }
}
