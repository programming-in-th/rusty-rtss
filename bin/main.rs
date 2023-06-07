#![feature(result_option_inspect)]
use std::sync::Arc;

use rusty_rtss::{app::App, sse::SsePublisher};

mod config;
mod listener;
mod payload;
mod publisher;
mod router;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

type Identifier = i32;

type Payload = payload::Payload;

type SharedState = Arc<App<SsePublisher<Identifier, Payload>>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match config::load_config() {
        Ok(x) => x,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    let pool = listener::get_pool_from_config(&config)
        .await
        .expect("Unable to create connection pool");
    log::info!("Connected to database");
    let listener = listener::get_listener_from_pool(&pool, &config)
        .await
        .expect("Unable to create listener from connection pool");
    log::info!("Listened to channel");

    let publisher = publisher::get_publisher();
    log::info!("Created publisher");

    let shared_state = Arc::new(App::new(listener, publisher).expect("Unable to create app"));
    log::info!("Created app");

    let router = router::get_router(shared_state);

    router::serve(router, &config).await.unwrap();
}
