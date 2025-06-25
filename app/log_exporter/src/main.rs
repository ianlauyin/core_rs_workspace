use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use core_ng::conf::load_conf;
use core_ng::kafka::consumer::ConsumerConfig;
use core_ng::kafka::consumer::MessageConsumer;
use core_ng::kafka::topic::Topic;
use core_ng::log;
use core_ng::log::ConsoleAppender;
use core_ng::shutdown::Shutdown;
use core_ng::task;
use core_ng::web::server::start_http_server;
use kafka::action_log_handler::ActionLogMessage;
use kafka::action_log_handler::action_log_message_handler;
use serde::Deserialize;
use web::upload;

pub mod kafka;
pub mod web;

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
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            topics: Topics {
                action: Topic::new("action-log-v2"),
            },
        }
    }
}

struct Topics {
    action: Topic<ActionLogMessage>,
}

#[tokio::main]
async fn main() -> Result<()> {
    log::init_with_action(ConsoleAppender);

    let config: AppConfig = load_conf()?;

    let shutdown = Shutdown::new();
    let http_signal = shutdown.subscribe();
    let consumer_signal = shutdown.subscribe();
    shutdown.listen();

    let state = Arc::new(AppState::default());

    let http_state = state.clone();
    task::spawn_task(async move {
        let app = Router::new();
        let app = app.merge(upload::routes());
        let app = app.with_state(http_state);
        start_http_server(app, http_signal).await
    });

    task::spawn_task(async move {
        let mut consumer = MessageConsumer::new(ConsumerConfig {
            bootstrap_servers: config.kafka_uri.clone(),
            ..Default::default()
        });
        consumer.add_bulk_handler(&state.topics.action, action_log_message_handler);
        consumer.start(state, consumer_signal).await
    });
    task::shutdown().await;

    Ok(())
}
