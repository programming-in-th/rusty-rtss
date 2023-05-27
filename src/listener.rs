use futures::Stream;

/// Make it easier to switch between database (e.g. postgres <-> supabase)
pub trait Listener: Send + Sync {
    type Data;
    type S: Stream<Item = Self::Data> + Unpin + Send;

    fn into_stream(self) -> Self::S;
}
