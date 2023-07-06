use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use futures_util::{SinkExt, Stream};
use tokio::sync::RwLock;

pub struct Replayer<T> {
    data: Arc<RwLock<Vec<T>>>,
    txs: Arc<RwLock<Vec<UnboundedSender<T>>>>,
}

impl<T> Replayer<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
            txs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_stream(&self) -> impl Stream<Item = T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let (mut tx, rx) = futures::channel::mpsc::unbounded::<T>();

        let c_tx = tx.clone();

        let data_ref = Arc::clone(&self.data);

        tokio::task::block_in_place(|| {
            self.txs.blocking_write().push(c_tx);
        });

        tokio::spawn(async move {
            let data = data_ref.read().await.clone();
            for data in data.into_iter() {
                let _ = tx.send(data.clone()).await;
            }
        });

        rx
    }

    pub async fn add_data(&self, data: T) {
        self.data.write().await.push(data);
    }
}
