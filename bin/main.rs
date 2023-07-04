#![feature(result_option_inspect)]

use repository::SubmisisonRepository;
use rusty_rtss::{app::App, postgres::PgConnector, sse::SsePublisher};

mod config;
mod connector;
mod payload;
mod publisher;
mod repository;
mod router;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

type Identifier = i32;

type Payload = payload::Payload;

#[derive(Clone)]
pub struct SharedState {
    app: App<PgConnector<Payload>, SsePublisher<Identifier, Payload>>,
    repository: SubmisisonRepository,
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let config = match config::load_config() {
        Ok(x) => x,
        Err(e) => {
            log::error!("Unable to load config: {e}");
            return;
        }
    };

    let pool = connector::get_pool_from_config(&config)
        .await
        .expect("Unable to create connection pool");
    log::info!("Connected to database");

    let connector = connector::get_connector_from_pool(&pool, &config)
        .await
        .expect("Unable to create listener from connection pool");
    log::info!("Listened to channel");

    let repository = repository::SubmisisonRepository::new(pool);
    log::info!("Created repository");

    let publisher = publisher::get_publisher();
    log::info!("Created publisher");

    let app = App::new(connector, publisher)
        .await
        .expect("Unable to create app");
    log::info!("Created app");

    let _app = app.clone();
    tokio::spawn(async move {
        if let Err(e) = _app.handle_connection().await {
            log::error!("Handle connection join error: {e:?}");
        }
    });

    let shared_state = SharedState { app, repository };

    let router = router::get_router(shared_state);

    router::serve(router, &config).await.unwrap();
}
