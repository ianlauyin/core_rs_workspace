use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use rdkafka::ClientConfig;
use rdkafka::Message as _;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::util::Timeout;
use serde::de::DeserializeOwned;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tracing::info;

use super::topic::Topic;
use crate::json::from_json;

pub struct Message<P: DeserializeOwned> {
    pub key: Option<String>,
    pub value: P,
}

trait MessageHandler<S>: Send + Sync {
    fn handle(
        &self,
        state: Arc<S>,
        messages: Vec<BorrowedMessage<'_>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}

impl<F, Fut, S> MessageHandler<S> for F
where
    F: Fn(Arc<S>, Vec<BorrowedMessage<'_>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + Sync + 'static,
{
    fn handle(
        &self,
        state: Arc<S>,
        messages: Vec<BorrowedMessage<'_>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(self(state, messages))
    }
}

pub struct ConsumerConfig {
    pub group_id: &'static str,
    pub bootstrap_servers: &'static str,
    pub poll_max_wait_time: Duration,
    pub poll_max_records: usize,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            group_id: env!("CARGO_PKG_NAME"),
            bootstrap_servers: "localhost:9092",
            poll_max_wait_time: Duration::from_secs(5),
            poll_max_records: 1000,
        }
    }
}

pub struct MessageConsumer<S> {
    config: ClientConfig,
    handlers: HashMap<&'static str, Box<dyn MessageHandler<S>>>,
    poll_max_wait_time: Duration,
    poll_max_records: usize,
}

impl<S> MessageConsumer<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(config: ConsumerConfig) -> Self {
        Self {
            config: ClientConfig::new()
                .set("group.id", config.group_id)
                .set("bootstrap.servers", config.bootstrap_servers)
                .set_log_level(RDKafkaLogLevel::Info)
                .to_owned(),
            handlers: HashMap::new(),
            poll_max_wait_time: config.poll_max_wait_time,
            poll_max_records: config.poll_max_records,
        }
    }

    pub fn add_handler<P, H, Fut>(&mut self, topic: &Topic<P>, handler: H)
    where
        P: DeserializeOwned + Send + Sync + 'static,
        H: Fn(Arc<S>, Message<P>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        let handler = move |state: Arc<S>, messages: Vec<BorrowedMessage<'_>>| {
            let mut message_groups: HashMap<Option<String>, Vec<Message<P>>> = HashMap::new();
            for message in messages {
                let key = message.key().map(|data| String::from_utf8_lossy(data).to_string());
                message_groups.entry(key).or_default().push(message.into());
            }
            let handler = handler.clone();
            async move {
                let mut handles = vec![];
                for (key, messages) in message_groups {
                    let state = Arc::clone(&state);
                    let handler = handler.clone();
                    if key.is_none() {
                        for message in messages {
                            let state = Arc::clone(&state);
                            handles.push(tokio::spawn(handler(state, message)));
                        }
                    } else {
                        handles.push(tokio::spawn(async move {
                            for message in messages {
                                let state = Arc::clone(&state);
                                handler(state, message).await?;
                            }
                            Ok(())
                        }));
                    }
                }
                for handle in handles {
                    handle.await??;
                }
                Ok(())
            }
        };

        self.handlers.insert(topic.name, Box::new(handler));
    }

    pub fn add_bulk_handler<P, H, Fut>(&mut self, topic: &Topic<P>, handler: H)
    where
        P: DeserializeOwned,
        H: Fn(Arc<S>, Vec<Message<P>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        let handler = move |state: Arc<S>, messages: Vec<BorrowedMessage<'_>>| {
            let messages = messages.into_iter().map(From::from).collect();
            handler(state, messages)
        };

        self.handlers.insert(topic.name, Box::new(handler));
    }

    pub async fn start(self, state: S, mut shutdown_signel: Receiver<()>) -> Result<()> {
        let state = Arc::new(state);

        let handlers = &self.handlers;
        let consumer: BaseConsumer = self.config.create()?;
        let topics: Vec<&str> = handlers.keys().cloned().collect();
        consumer.subscribe(&topics)?;

        info!("kakfa consumer started, topics={:?}", topics);

        loop {
            let messages = poll_messages(&consumer, self.poll_max_wait_time, self.poll_max_records)?;

            if !messages.is_empty() {
                let mut handles: Vec<JoinHandle<Result<()>>> = vec![];

                for (topic, messages) in messages {
                    if let Some(handler) = handlers.get(topic.as_str()) {
                        let state = state.clone();
                        handles.push(tokio::spawn(handler.handle(state, messages)));
                    }
                    // subscribed topic must have a handler
                }

                for handle in handles {
                    handle.await??;
                }

                consumer.commit_consumer_state(CommitMode::Async)?;
            }

            if shutdown_signel.try_recv().is_ok() {
                info!("kakfa consumer stopped, topics={:?}", topics);
                return Ok(());
            }
        }
    }
}

impl<P: DeserializeOwned> From<BorrowedMessage<'_>> for Message<P> {
    fn from(message: BorrowedMessage<'_>) -> Message<P> {
        let key = message.key().map(|data| String::from_utf8_lossy(data).to_string());
        let value = message.payload().map(|data| String::from_utf8_lossy(data).to_string());

        let value = value.map(|v| from_json(&v).unwrap()).unwrap();
        Message { key, value }
    }
}

fn poll_messages(
    consumer: &'_ BaseConsumer,
    max_wait_time: Duration,
    max_records: usize,
) -> Result<HashMap<String, Vec<BorrowedMessage<'_>>>> {
    let mut messages: HashMap<String, Vec<BorrowedMessage<'_>>> = HashMap::new();
    let start_time = Instant::now();
    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= max_wait_time {
            break;
        }

        if messages.len() >= max_records {
            break;
        }

        if let Some(result) = consumer.poll(Timeout::After(max_wait_time.saturating_sub(elapsed))) {
            let message = result?;
            let topic = message.topic().to_owned();
            messages.entry(topic).or_default().push(message);
        }
    }
    Ok(messages)
}
