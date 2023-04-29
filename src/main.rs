#![feature(type_alias_impl_trait)]
use std::sync::Arc;

use axum::{routing::get, Router, Server};
use futures::channel::mpsc::UnboundedSender;

use rusty_rtss::{
    postgres::{PgListener, PgListenerConfig},
    rtss::App,
    sse::SsePublisher,
};

struct Payload;

type SharedState = Arc<App<String, UnboundedSender<Payload>, Payload>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pg_listener = sqlx::postgres::PgListener::connect("localhost")
        .await
        .unwrap();

    let listener = PgListener::connect(&PgListenerConfig {
        channels: vec!["submit"],
        url: std::env::var("DB_CONNECTION_STRING")
            .expect("DB connection is not provided")
            .as_str(),
    });

    let publisher = SsePublisher::new();

    let shared_state = Arc::new(App::new(listener, publisher));

    let app = Router::new()
        .route("/:submission_id", get(handler))
        .with_state(shared_state);

    Server::bind(&"0.0.0.0".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
