use std::sync::Arc;

use anyhow::Result;
use core_ng::kafka::consumer::ConsumerConfig;
use core_ng::kafka::consumer::Message;
use core_ng::kafka::consumer::MessageConsumer;
use core_ng::kafka::topic::Topic;
use core_ng::shutdown::Shutdown;
use serde::Deserialize;
use serde::Serialize;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Serialize, Deserialize, Debug)]
struct TestMessage {
    name: String,
}

struct State {
    topics: Topics,
}

struct Topics {
    test: Topic<TestMessage>,
    test_single: Topic<TestMessage>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(LevelFilter::INFO),
        )
        .init();

    let state = State {
        topics: Topics {
            test: Topic::new("test"),
            test_single: Topic::new("test_single"),
        },
    };

    let shutdown = Shutdown::new();
    let signal = shutdown.subscribe();
    shutdown.listen();

    let mut consumer = MessageConsumer::new(ConsumerConfig {
        group_id: "log-exporter",
        bootstrap_servers: "dev.internal:9092",
        ..ConsumerConfig::default()
    });

    consumer.add_bulk_handler(&state.topics.test, handler);
    consumer.add_handler(&state.topics.test_single, handler_single);
    consumer.start(state, signal).await
}

async fn handler(_state: Arc<State>, messages: Vec<Message<TestMessage>>) -> Result<()> {
    println!("count => {}", messages.len());
    for message in messages {
        println!("Received message: {}", message.value.name);
    }
    Ok(())
}

async fn handler_single(_state: Arc<State>, message: Message<TestMessage>) -> Result<()> {
    println!("Received single message: {}", message.value.name);
    Ok(())
}
