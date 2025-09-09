use std::sync::Arc;

use axum::Router;
use chrono::FixedOffset;
use chrono::NaiveTime;
use framework::asset::asset_path;
use framework::exception::Exception;
use framework::json;
use framework::kafka::consumer::ConsumerConfig;
use framework::kafka::consumer::MessageConsumer;
use framework::kafka::topic::Topic;
use framework::log;
use framework::log::ConsoleAppender;
use framework::schedule::Scheduler;
use framework::shutdown::Shutdown;
use framework::task;
use framework::web::server::HttpServerConfig;
use framework::web::server::start_http_server;
use serde::Deserialize;

mod kibana;

#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    kafka_uri: String,
    kibana_uri: String,
}

pub struct AppState {
    topics: Topics,
}

impl AppState {
    fn new(config: &AppConfig) -> Result<Self, Exception> {
        Ok(AppState {
            topics: Topics {
                // action: Topic::new("action-log-v2"),
                // event: Topic::new("event"),
            },
        })
    }
}

struct Topics {
    // action: Topic<ActionLogMessage>,
    // event: Topic<EventMessage>,
}

#[tokio::main]
async fn main() -> Result<(), Exception> {
    log::init_with_action(ConsoleAppender);

    let config: AppConfig = json::load_file(&asset_path("assets/conf.json")?)?;

    let shutdown = Shutdown::new();
    // let consumer_signal = shutdown.subscribe();
    shutdown.listen();

    // let state = Arc::new(AppState::new(&config)?);
    // let consumer_state = state.clone();

    task::spawn_action("import_kibana_objects", async { Ok(()) });

    // task::spawn_task(async move {
    //     let mut consumer = MessageConsumer::new(ConsumerConfig {
    //         bootstrap_servers: config.kafka_uri.clone(),
    //         ..Default::default()
    //     });
    //     consumer.add_bulk_handler(&consumer_state.topics.action, action_log_message_handler);
    //     consumer.add_bulk_handler(&consumer_state.topics.event, event_message_handler);
    //     consumer.start(consumer_state, consumer_signal).await
    // });

    task::shutdown().await;

    Ok(())
}
