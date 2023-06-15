use std::{marker::PhantomData, time::Duration};

use sqlx::PgPool;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::listener::Connector;

use super::{builder::PgConnectorBuilder, PgListener};

pub struct PgConnector<P> {
    connection_method: ConnectionMethod,
    last_attempt_handle: Mutex<Option<JoinHandle<()>>>,
    listen_channels: Vec<String>,
    _payload: PhantomData<P>,
}

enum ConnectionMethod {
    Url(String),
    Pool(PgPool),
}

impl<P> PgConnector<P> {
    fn new(connection_method: ConnectionMethod, listen_channels: Vec<String>) -> Self {
        Self {
            connection_method,
            listen_channels,
            last_attempt_handle: Default::default(),
            _payload: PhantomData,
        }
    }

    pub fn builder() -> PgConnectorBuilder<P> {
        PgConnectorBuilder::new()
    }

    pub fn from_pool(pool: PgPool, channels: Vec<String>) -> Self {
        Self::new(ConnectionMethod::Pool(pool), channels)
    }

    pub fn from_url(url: String, channels: Vec<String>) -> Self {
        Self::new(ConnectionMethod::Url(url), channels)
    }
}

#[async_trait::async_trait]
impl<P> Connector for PgConnector<P>
where
    P: Send + Sync + From<sqlx::postgres::PgNotification>,
{
    type Listener = PgListener<P>;

    async fn connect(&self) -> Option<Self::Listener> {
        let mut listener = loop {
            let mut lock = self.last_attempt_handle.lock().await;

            if let Some(handle) = lock.take() {
                if let Err(e) = handle.await {
                    log::error!("Reconnect join error: {e:?}");
                }
            }

            let new_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(180)).await;
            });

            *lock = Some(new_handle);
            drop(lock);

            log::trace!("Trying to connect to database...");
            if let Some(listener) = match &self.connection_method {
                ConnectionMethod::Url(url) => sqlx::postgres::PgListener::connect(url).await.ok(),
                ConnectionMethod::Pool(pool) => {
                    sqlx::postgres::PgListener::connect_with(pool).await.ok()
                }
            } {
                break listener;
            }
        };

        let channels: Vec<&str> = self.listen_channels.iter().map(|x| x.as_str()).collect();

        listener.listen_all(channels).await.ok()?;

        Some(PgListener::new(listener))
    }
}
