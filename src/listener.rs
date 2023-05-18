use futures::Stream;

/// Make it easier to switch between database (e.g. postgres <-> supabase)
pub trait Listener: Send + Sync {
    type Payload;
    type Identifier;
    type S: Stream<Item = (Self::Identifier, Self::Payload)> + Unpin + Send;

    fn into_stream(self) -> Self::S;
}
