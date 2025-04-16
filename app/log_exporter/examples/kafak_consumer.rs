use std::sync::Arc;

use anyhow::Result;
use core_ng::kafka::consumer::ConsumerConfig;
use core_ng::kafka::consumer::Message;
use core_ng::kafka::consumer::MessageConsumer;
use core_ng::kafka::producer::Producer;
use core_ng::kafka::producer::ProducerConfig;
use core_ng::kafka::topic::Topic;
use core_ng::log;
use core_ng::log::ConsoleAppender;
use core_ng::shutdown::Shutdown;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

#[derive(Serialize, Deserialize, Debug)]
struct TestMessage {
    name: String,
}

struct State {
    topics: Topics,
    producer: Producer,
}

struct Topics {
    test_single: Topic<TestMessage>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    log::init(ConsoleAppender);

    let state = State {
        topics: Topics {
            test_single: Topic::new("test_single"),
        },
        producer: Producer::new(ProducerConfig {
            bootstrap_servers: "dev.internal:9092",
        }),
    };

    let shutdown = Shutdown::new();
    let signal = shutdown.subscribe();
    shutdown.listen();

    let mut consumer = MessageConsumer::new(ConsumerConfig {
        group_id: "log-exporter",
        bootstrap_servers: "dev.internal:9092",
    });

    consumer.add_handler(&state.topics.test_single, handler_single);
    consumer.start(state, signal).await
}

async fn handler_single(state: Arc<State>, message: Message<TestMessage>) -> Result<()> {
    println!("Received single message: {}", message.payload()?.name);

    if let Some(ref key) = message.key {
        if key == "1" {
            let value = message.payload()?;
            state
                .producer
                .send(&state.topics.test_single, Some("xxx".to_string()), &value)
                .await?;
            warn!("test: {value:?}");
        } else if key == "xxx" {
            warn!("test");
        }
    }

    Ok(())
}
