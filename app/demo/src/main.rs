use std::sync::Arc;

use axum::Router;
use framework::exception::Exception;
use framework::log;
use framework::log::ConsoleAppender;
use framework::shutdown::Shutdown;
use framework::web::server::HttpServerConfig;
use framework::web::server::start_http_server;
use serde::Deserialize;

mod web;

#[derive(Debug, Deserialize, Clone)]
struct AppConfig {}

pub struct AppState {}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    log::init_with_action(ConsoleAppender);

    let shutdown = Shutdown::new();
    let signal = shutdown.subscribe();
    shutdown.listen();

    let state = Arc::new(AppState {});

    let app = Router::new();
    let app = app.merge(web::routes());
    let app = app.with_state(state);
    start_http_server(app, signal, HttpServerConfig::default()).await
}
