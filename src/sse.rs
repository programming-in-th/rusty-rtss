use std::{fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};

use axum::response::sse::Event;
use futures::{channel::mpsc::UnboundedSender, SinkExt};

use dashmap::DashMap;

use crate::publisher::Publisher;

pub struct SsePublisher<I, P> {
    connections: Arc<DashMap<I, UnboundedSender<Event>>>,
    _payload: PhantomData<P>,
}

pub trait Identifiable {
    type Identifier;

    fn id(&self) -> Self::Identifier;
}

#[async_trait::async_trait]
impl<I, P> Publisher for SsePublisher<I, P>
where
    P: Send + Sync + Into<Event> + Debug + Identifiable<Identifier = I>,
    I: Send + Sync + std::hash::Hash + Eq + Copy + 'static + Debug,
{
    type Subscriber = SseSubscriber<I>;
    type PublishData = P;

    fn add_subscriber(&self, subscriber: Self::Subscriber) {
        log::info!("Received add subscriber");

        let SseSubscriber { id, writer } = subscriber;

        let connections = Arc::clone(&self.connections);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            log::debug!("timeout: {id:?}");
            connections.remove(&id);
        });

        self.connections.insert(id, writer);
    }

    async fn publish(&self, data: Self::PublishData) {
        log::debug!("Received add publish");

        let id = data.id();

        let mut remove_id = None;

        if let Some(conns) = self.connections.get(&id) {
            // Sender is cloneable
            let mut writer = conns.value().clone();

            log::debug!("found subscriber, publishing: {data:?}");

            if let Err(e) = writer.send(<P as Into<Event>>::into(data)).await {
                log::warn!("unable to publish: {e:?}");
                remove_id = Some(id);
                writer.close().await.unwrap();
            }
        }

        if let Some(remove_id) = remove_id {
            self.connections.remove(&remove_id);
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

pub struct SseSubscriber<I> {
    id: I,
    writer: UnboundedSender<Event>,
}

impl<I> SseSubscriber<I> {
    pub fn new(id: I, writer: UnboundedSender<Event>) -> Self {
        Self { id, writer }
    }
}
