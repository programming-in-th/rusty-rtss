use super::event::{Listener, Publisher};

use futures::{channel::mpsc::UnboundedSender, SinkExt};
use futures_util::StreamExt;
use tokio::task::JoinHandle;

enum Event<I, W, T> {
    AddSubscriber(I, W),
    Publish(I, T),
}

pub struct App<I, W, T> {
    event_writer: UnboundedSender<Event<I, W, T>>,
    _handle: JoinHandle<()>,
}

impl<I, W, T> App<I, W, T> {
    pub fn new<L, P>(listener: L, mut publisher: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        L: Listener<Payload = T, Identifier = I> + 'static,
        P: Publisher<Payload = T, Identifier = I, Writer = W> + 'static,
        I: Send + Sync + 'static,
        W: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let (event_writer, mut event_reader) = futures::channel::mpsc::unbounded();

        let cloned_writer = event_writer.clone();

        let _forwarder = tokio::spawn(async move {
            listener
                .into_stream()
                .map(|(id, payload)| Ok(Event::Publish(id, payload)))
                .forward(cloned_writer)
                .await
        });

        let _event_processor = tokio::spawn(async move {
            log::info!("Spawning event hub");
            while let Some(event) = event_reader.next().await {
                match event {
                    Event::AddSubscriber(id, writer) => publisher.add_subscriber(id, writer),
                    Event::Publish(id, payload) => publisher.publish(&id, payload).await,
                }
            }
        });

        // app should die when either handle die
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = _forwarder => {

                }
                _ = _event_processor => {

                }
            }
        });

        Ok(App {
            event_writer,
            _handle: handle,
        })
    }

    /// Push add subscriber event to message queue.
    pub async fn add_subscriber(&self, id: I, writer: W) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self
            .event_writer
            .clone()
            .send(Event::AddSubscriber(id, writer))
            .await?)
    }
}
