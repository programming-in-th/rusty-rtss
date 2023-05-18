use std::{fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};

use axum::response::sse::Event;
use futures::{channel::mpsc::UnboundedSender, SinkExt};

use dashmap::DashMap;

use crate::publisher::Publisher;

pub struct SsePublisher<I, P> {
    connections: Arc<DashMap<I, UnboundedSender<Event>>>,
    _payload: PhantomData<P>,
}

#[async_trait::async_trait]
impl<I, P> Publisher for SsePublisher<I, P>
where
    P: Send + Sync + Into<Event> + Debug,
    I: Send + Sync + std::hash::Hash + Eq + Copy + 'static + Debug,
{
    type Payload = P;
    type Identifier = I;
    type Target = Event;
    type Writer = UnboundedSender<Event>;

    fn add_subscriber(&self, id: Self::Identifier, writer: Self::Writer) {
        log::info!("Received add subscriber");

        let connections = Arc::clone(&self.connections);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            log::debug!("timeout: {id:?}");
            connections.remove(&id);
        });

        self.connections.insert(id, writer);
    }

    async fn publish(&self, id: &Self::Identifier, payload: Self::Payload) {
        log::info!("Received add publish");
        if let Some(conns) = self.connections.get(id) {
            // Sender is cloneable
            let mut writer = conns.value().clone();

            log::debug!("found subscriber, publishing: {payload:?}");

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
            connections: Arc::new(Default::default()),
            _payload: Default::default(),
        }
    }
}
