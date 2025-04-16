use anyhow::Result;
use axum::Router;
use core_ng::log;
use core_ng::log::ConsoleAppender;
use core_ng::shutdown::Shutdown;
use core_ng::task;
use core_ng::web::server::start_http_server;
use web::upload;

pub mod web;

#[derive(Clone)]
pub struct ApiState {}

#[tokio::main]
async fn main() -> Result<()> {
    log::init(ConsoleAppender);

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
