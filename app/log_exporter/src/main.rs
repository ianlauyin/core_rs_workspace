use anyhow::Result;
use axum::Router;
use core_ng::shutdown::Shutdown;
use core_ng::task;
use core_ng::web::server::start_http_server;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use web::upload;

pub mod web;

#[derive(Clone)]
pub struct ApiState {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_filter(LevelFilter::INFO),
        )
        .init();

    let mut args = std::env::args();
    if let Some(conf) = args.nth(1) {
        println!("conf: {}", conf);
    }

    let shutdown = Shutdown::new();
    let signal = shutdown.subscribe();
    shutdown.listen();

    let state = ApiState {};

    let app = Router::new();
    let app = app.merge(upload::routes());
    let app = app.with_state(state);

    start_http_server(app, signal).await?;
    task::shutdown().await;

    Ok(())
}
