/// Make it easier to switch between client implementation (e.g. ws <-> sse)
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    type Subscriber;
    type PublishData;

    fn add_subscriber(&self, subscriber: Self::Subscriber);

    async fn publish(&self, data: Self::PublishData);
}
