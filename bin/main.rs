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
    app::App,
    postgres::{PgListener, PgListenerConfig},
    sse::{SsePublisher, SseSubscriber},
};
use tower_http::cors::Any;

mod payload {
    use axum::response::sse::Event;
    use rusty_rtss::sse::Identifiable;
    use serde::{Deserialize, Serialize};
    use sqlx::postgres::PgNotification;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Payload {
        pub id: i32,
        pub groups: Vec<Group>,
        pub score: i32,
        pub status: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Group {
        score: f64,
        full_score: f64,
        submission_id: String,
        group_index: i32,
        run_result: Vec<RunResult>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct RunResult {
        submission_id: String,
        test_index: i32,
        status: String,
        time_usage: f64,
        memory_usage: i32,
        score: f64,
        message: String,
    }

    impl Identifiable for Payload {
        type Identifier = i32;

        fn id(&self) -> Self::Identifier {
            self.id
        }
    }

    impl From<Payload> for Event {
        fn from(value: Payload) -> Self {
            Event::default()
                .json_data(value)
                .expect("unable to serialize payload")
        }
    }

    impl From<PgNotification> for Payload {
        fn from(value: PgNotification) -> Self {
            serde_json::from_str(value.payload()).unwrap()
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

    let subscriber = SseSubscriber::new(submission_id, tx);

    let _ = shared_state
        .add_subscriber(subscriber)
        .await
        .inspect_err(|err| {
            log::warn!("error while adding subscriber: {err:?}");
        });

    let rx = rx.map(Result::<Event, Error>::Ok);

    Sse::new(rx)
}

async fn healthz() -> impl IntoResponse {
    "OK"
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = std::env::var("RTSS_PORT").expect("`RTSS_PORT` is not provided");

    let addr = format!("0.0.0.0:{port}").parse().unwrap();

    log::info!("Connection to database");
    let listener = PgListener::<Payload>::connect(PgListenerConfig {
        channels: vec!["update"],
        url: std::env::var("DB_CONNECTION_URI")
            .expect("`DB_CONNECTION_URI` is not provided")
            .as_str(),
    })
    .await
    .expect("unable to connect to database");

    log::info!("Creating publisher");
    let publisher = SsePublisher::new();

    log::info!("Creating app");
    let shared_state = Arc::new(App::new(listener, publisher).expect("unable to create app"));

    let cors = tower_http::cors::CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let app = Router::new()
        .route("/:submission_id", get(handler))
        .route("/", get(healthz))
        .layer(cors)
        .with_state(shared_state);

    log::info!("Serving on {addr:?}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
