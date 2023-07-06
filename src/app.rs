use std::{sync::Arc, time::Duration};

use futures_util::Stream;

use crate::cfg::AppConfig;

use crate::replayer::Replayer;

pub struct App {
    replayers: Arc<dashmap::DashMap<i32, Arc<Replayer<crate::Payload>>>>,
    config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            replayers: Arc::new(dashmap::DashMap::new()),
            config,
        }
    }

    pub fn get_stream(&self, id: i32) -> impl Stream<Item = crate::Payload> {
        self.get_replayer(id).get_stream()
    }

    pub async fn write_to_stream(&self, id: i32, data: crate::Payload) {
        self.get_replayer(id).add_data(data).await
    }

    fn get_replayer(&self, id: i32) -> Arc<Replayer<crate::Payload>> {
        if let Some(replayer) = self.replayers.get(&id) {
            Arc::clone(&replayer)
        } else {
            let replayer = Arc::new(Replayer::new());

            self.replayers.insert(id, Arc::clone(&replayer));

            let replayers = Arc::clone(&self.replayers);

            let replay_duration = self.config.replay_duration;

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(replay_duration)).await;
                replayers.remove(&id);
            });

            replayer
        }
    }
}
