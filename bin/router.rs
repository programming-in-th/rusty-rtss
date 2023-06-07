use axum::{
    extract::{Path, State},
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router,
};
use futures::channel::mpsc::unbounded;
use futures_util::{SinkExt, StreamExt};
use rusty_rtss::sse::SseSubscriber;
use tower_http::cors::Any;

use crate::{config::Config, Result};

use super::SharedState;

async fn healthz() -> impl IntoResponse {
    "OK"
}

async fn handler(
    Path(submission_id): Path<i32>,
    State(shared_state): State<SharedState>,
) -> impl IntoResponse {
    let (mut tx, rx) = unbounded::<Event>();

    let subscriber = SseSubscriber::new(submission_id, tx.clone());

    tokio::spawn(async move {
        let repository = shared_state.repository.clone();

        match repository.get_submission_by_id(submission_id).await {
            Err(e) => {
                log::warn!("Unable to received payload: {e}");
            }
            Ok(payload) => {
                if let Err(e) = tx.send(payload.into()).await {
                    log::warn!("Unable to send payload: {e}");
                }
            }
        };
    });

    let _ = shared_state
        .app
        .add_subscriber(subscriber)
        .await
        .inspect_err(|err| {
            log::warn!("error while adding subscriber: {err:?}");
        });

    let rx = rx.map(Result::<Event>::Ok);

    Sse::new(rx)
}

pub fn get_router(shared_state: SharedState) -> Router {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any);

    Router::new()
        .route("/:submission_id", get(handler))
        .route("/", get(healthz))
        .layer(cors)
        .with_state(shared_state)
}

pub async fn serve(router: Router, config: &Config) -> Result<()> {
    let addr = format!("{}:{}", config.axum.host, config.axum.port).parse()?;

    Ok(axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?)
}
