use std::marker::PhantomData;

use axum::response::sse::Event;
use futures::{channel::mpsc::UnboundedSender, SinkExt};

use dashmap::DashMap;

use crate::event::Publisher;

pub struct SsePublisher<I, P> {
    connections: DashMap<I, UnboundedSender<Event>>,
    _payload: PhantomData<P>,
    _id: PhantomData<I>,
}

#[async_trait::async_trait]
impl<I, P> Publisher for SsePublisher<I, P>
where
    P: Send + Sync + Into<Event>,
    I: Send + Sync + std::hash::Hash + Eq,
{
    type Payload = P;
    type Identifier = I;
    type Target = Event;
    type Writer = UnboundedSender<Event>;

    fn add_subscriber(&mut self, id: Self::Identifier, writer: Self::Writer) {
        log::info!("Received add subscriber");

        self.connections.insert(id, writer);

        // TODO handle timeout
    }

    async fn publish(&self, id: &Self::Identifier, payload: Self::Payload) {
        log::info!("Received add publish");
        if let Some(conns) = self.connections.get(id) {
            // Sender is cloneable
            let mut writer = conns.value().clone();

            if let Err(e) = writer.send(<P as Into<Event>>::into(payload)).await {
                log::warn!("unable to publish: {e:?}");
                self.connections.remove(id);
            }
        }
    }
}

impl<I, P> SsePublisher<I, P> {
    pub fn new() -> Self
    where
        I: Eq + std::hash::Hash,
    {
        SsePublisher {
            connections: Default::default(),
            _payload: Default::default(),
            _id: Default::default(),
        }
    }
}
