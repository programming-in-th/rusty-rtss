#![feature(result_option_inspect)]
use std::sync::Arc;

use futures_util::StreamExt;

use axum::{
    extract::{Path, State},
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router, Server,
};
use futures::channel::mpsc::unbounded;

use rusty_rtss::{
    postgres::{PgListener, PgListenerConfig},
    rtss::App,
    sse::SsePublisher,
};

mod payload {
    use axum::response::sse::Event;
    use rusty_rtss::postgres::Identifiable;
    use serde::{Deserialize, Serialize};
    use sqlx::postgres::PgNotification;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Payload {
        pub id: i32,
        pub payload: String,
    }

    impl Identifiable for Payload {
        type Identifier = i32;

        fn from(input: PgNotification) -> (i32, Self) {
            let v: Payload =
                serde_json::from_str(input.payload()).expect("unable to deserialize json");

            (v.id, v)
        }
    }

    impl Into<Event> for Payload {
        fn into(self) -> Event {
            Event::default()
                .json_data(self)
                .expect("unable to serialize payload")
        }
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

type Identifier = i32;

type Payload = payload::Payload;

type SharedState = Arc<App<SsePublisher<Identifier, Payload>>>;

async fn handler(
    Path(submission_id): Path<i32>,
    State(shared_state): State<SharedState>,
) -> impl IntoResponse {
    let (tx, rx) = unbounded::<Event>();

    let _ = shared_state
        .add_subscriber(submission_id, tx)
        .await
        .inspect_err(|err| {
            log::warn!("error while adding subscriber: {err:?}");
        });

    let rx = rx.map(Result::<Event, Error>::Ok);

    let sse = Sse::new(rx);

    sse
}

async fn healthz() -> impl IntoResponse {
    "OK"
}

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("Connection to database");
    let listener = PgListener::<Identifier, Payload>::connect(PgListenerConfig {
        channels: vec!["update"],
        url: std::env::var("DB_CONNECTION_URI")
            .expect("DB connection is not provided")
            .as_str(),
    })
    .await
    .expect("unable to connect to database");

    log::info!("Creating publisher");
    let publisher = SsePublisher::new();

    log::info!("Creating app");
    let shared_state = Arc::new(App::new(listener, publisher).expect("unable to create app"));

    let app = Router::new()
        .route("/:submission_id", get(handler))
        .route("/", get(healthz))
        .with_state(shared_state);

    log::info!("Serving on 0.0.0.0:3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
