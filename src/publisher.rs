use futures_util::Sink;

/// Make it easier to switch between client implementation (e.g. ws <-> sse)
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    type Payload: Into<Self::Target>;
    type Identifier;
    type Target;
    type Writer: Sink<Self::Target>;

    fn add_subscriber(&self, id: Self::Identifier, writer: Self::Writer);

    async fn publish(&self, id: &Self::Identifier, payload: Self::Payload);
}
