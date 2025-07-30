use std::sync::Arc;

use axum::Router;
use framework::conf::load_conf;
use framework::exception::Exception;
use framework::kafka::producer::Producer;
use framework::kafka::producer::ProducerConfig;
use framework::kafka::topic::Topic;
use framework::log;
use framework::log::ConsoleAppender;
use framework::shutdown::Shutdown;
use framework::web::server::HttpServerConfig;
use framework::web::server::start_http_server;
use kafka::EventMessage;
use serde::Deserialize;

mod kafka;
mod web;

#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    kafka_uri: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            kafka_uri: "dev.internal:9092".to_owned(),
        }
    }
}

pub struct AppState {
    topics: Topics,
    producer: Producer,
}

impl AppState {
    fn new(config: &AppConfig) -> Result<Self, Exception> {
        Ok(AppState {
            topics: Topics {
                event: Topic::new("event"),
            },
            producer: Producer::new(ProducerConfig {
                bootstrap_servers: config.kafka_uri.to_string(),
            }),
        })
    }
}

struct Topics {
    event: Topic<EventMessage>,
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    log::init_with_action(ConsoleAppender);

    let config: AppConfig = load_conf()?;

    let shutdown = Shutdown::new();
    let signal = shutdown.subscribe();
    shutdown.listen();

    let state = Arc::new(AppState::new(&config)?);

    let app = Router::new();
    let app = app.merge(web::routes());
    let app = app.with_state(state);
    start_http_server(app, signal, HttpServerConfig::default()).await
}
