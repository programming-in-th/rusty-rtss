use std::marker::PhantomData;

use futures::channel::mpsc::UnboundedSender;

use super::event::Publisher;

pub struct SsePublisher<P> {
    _payload: PhantomData<P>,
}

#[async_trait::async_trait]
impl<P> Publisher for SsePublisher<P>
where
    P: Send + Sync,
{
    type Payload = P;
    type Identifier = String;
    type Writer = UnboundedSender<Self::Payload>;

    fn add_subscriber(&mut self, _id: &Self::Identifier, _writer: Self::Writer) {
        log::info!("Received add subscriber: {_id}");
        // todo!()
    }

    async fn publish(&self, _id: &Self::Identifier, _payload: &Self::Payload) {
        log::info!("Received add publish: {_id}");
        // todo!()
    }
}

impl<P> SsePublisher<P> {
    pub fn new() -> Self {
        SsePublisher {
            _payload: Default::default(),
        }
    }
}

impl<P> Default for SsePublisher<P> {
    fn default() -> Self {
        Self::new()
    }
}
