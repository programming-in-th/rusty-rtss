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
        L: Listener<Payload = T, Identifier = I>,
        P: Publisher<Payload = T, Identifier = I, Writer = W> + 'static,
        I: Send + Sync + 'static,
        W: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let (event_writer, mut event_reader) = futures::channel::mpsc::unbounded();

        let cloned_writer = event_writer.clone();

        futures::executor::block_on(async move {
            listener
                .into_stream()
                .map(|(id, payload)| Ok(Event::Publish(id, payload)))
                .forward(cloned_writer)
                .await
        })?;

        let handle = tokio::spawn(async move {
            while let Some(event) = event_reader.next().await {
                match event {
                    Event::AddSubscriber(id, writer) => publisher.add_subscriber(&id, writer),
                    Event::Publish(id, payload) => publisher.publish(&id, &payload).await,
                }
            }
        });

        Ok(App {
            event_writer,
            _handle: handle,
        })
    }

    pub async fn add_subscriber(
        &mut self,
        id: I,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self
            .event_writer
            .send(Event::AddSubscriber(id, writer))
            .await?)
    }
}
