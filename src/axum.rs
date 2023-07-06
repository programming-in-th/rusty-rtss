use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Sse},
    routing::get,
};
use futures_util::StreamExt;

use crate::{app::App, cfg::AxumConfig};

pub async fn handle_sse(State(app): State<Arc<App>>, Path(id): Path<i32>) -> impl IntoResponse {
    tracing::info!("Recv subscriber: {id}");

    let stream = app.get_stream(id).map(Into::into);

    Sse::new(stream)
}

pub async fn health_check() -> impl IntoResponse {
    "OK"
}

pub async fn serve(
    router: axum::Router,
    config: AxumConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = &format!("{}:{}", config.listen_host, config.listen_port).parse()?;

    axum::Server::bind(addr)
        .serve(router.into_make_service())
        .await
        .map_err(Into::into)
}

pub fn get_router(state: Arc<App>) -> axum::Router {
    use tower_http::cors::Any;

    let cors_layer = tower_http::cors::CorsLayer::new()
        .allow_headers(Any)
        .allow_origin(Any)
        .allow_methods(Any);

    axum::Router::new()
        .route("/:id", get(handle_sse))
        .route("/", get(health_check))
        .with_state(state)
        .layer(cors_layer)
}
