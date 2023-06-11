use futures::Stream;

/// Make it easier to switch between database (e.g. postgres <-> supabase)
pub trait Listener: Send + Sync {
    type Data;
    type S: Stream<Item = Self::Data> + Unpin + Send;

    fn into_stream(self) -> Self::S;
}

#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    type Listener: Listener;

    /// `None` indicates that there will be no connection continue
    /// the default implementation is also `None`
    async fn connect(&self) -> Option<Self::Listener> {
        None
    }
}
