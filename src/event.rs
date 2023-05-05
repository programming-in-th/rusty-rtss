use futures::{Sink, Stream};

/// Make it easier to switch between database (e.g. postgres <-> supabase)
pub trait Listener: Send + Sync {
    type Payload;
    type Identifier;
    type S: Stream<Item = (Self::Identifier, Self::Payload)> + Unpin + Send;

    fn into_stream(self) -> Self::S;
}

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
