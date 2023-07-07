mod app;
mod axum;
mod cfg;
mod payload;
mod replayer;
mod rmq;

use std::sync::Arc;

use cfg::Config;
use futures_util::StreamExt;
use payload::Payload;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder().with_line_number(true).with_max_level(tracing::Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set trace subscriber");

    let Config {
        app_config,
        rmq_config,
        axum_config,
    } = match cfg::read_config() {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("Unable to read config: {e}");
            return;
        }
    };
    tracing::info!("Read config successfully");

    let app = Arc::new(app::App::new(app_config));
    let cloned_app = Arc::clone(&app);

    let mut stream = rmq::get_stream(rmq_config)
        .await
        .expect("Unable to get stream");
    tracing::info!("Created stream");

    tokio::spawn(async move {
        let app = cloned_app;
        while let Some(x) = stream.next().await {
            app.write_to_stream(x.id, x).await
        }
    });
    tracing::info!("Start message relay");

    let router = axum::get_router(Arc::clone(&app));

    tracing::info!("Starting http handler");
    if let Err(e) = axum::serve(router, axum_config).await {
        tracing::info!("Server closed: {e}");
    }
}
