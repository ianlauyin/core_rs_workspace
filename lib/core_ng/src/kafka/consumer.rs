use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::DateTime;
use chrono::SecondsFormat;
use chrono::Utc;
use rdkafka::ClientConfig;
use rdkafka::Message as _;
use rdkafka::Timestamp;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::Headers;
use rdkafka::util::Timeout;
use serde::de::DeserializeOwned;
use tokio::sync::broadcast::Receiver;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::info;

use super::topic::Topic;
use crate::env;
use crate::json::from_json;
use crate::log;
use crate::time::duration;

pub struct Message<T: DeserializeOwned> {
    pub key: Option<String>,
    payload: String,
    pub headers: HashMap<String, String>,
    pub timestamp: Option<DateTime<Utc>>,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned> Message<T> {
    pub fn payload(&self) -> Result<T> {
        from_json(&self.payload)
    }
}

trait MessageHandler<S> {
    fn handle(&self, state: Arc<S>, messages: BorrowedMessage<'_>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<F, Fut, S> MessageHandler<S> for F
where
    F: Fn(Arc<S>, BorrowedMessage<'_>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + Sync + 'static,
{
    fn handle(&self, state: Arc<S>, messages: BorrowedMessage<'_>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(self(state, messages))
    }
}

pub struct ConsumerConfig {
    pub group_id: &'static str,
    pub bootstrap_servers: &'static str,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            group_id: env::APP_NAME,
            bootstrap_servers: "localhost:9092",
        }
    }
}

pub struct MessageConsumer<S> {
    config: ClientConfig,
    handlers: HashMap<&'static str, Box<dyn MessageHandler<S>>>,
    tasks: TaskTracker,
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
                .set("enable.auto.commit", "true")
                .set_log_level(RDKafkaLogLevel::Info)
                .to_owned(),
            handlers: HashMap::new(),
            tasks: TaskTracker::new(),
        }
    }

    pub fn add_handler<H, Fut, M>(&mut self, topic: &Topic<M>, handler: H)
    where
        H: Fn(Arc<S>, Message<M>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + Sync + 'static,
        M: DeserializeOwned + Send + Sync + 'static,
    {
        let topic = topic.name;
        let handler = move |state: Arc<S>, message: BorrowedMessage<'_>| {
            let handler = handler.clone();
            let message = From::from(message);
            handle_message(topic, message, handler, state)
        };

        self.handlers.insert(topic, Box::new(handler));
    }

    pub async fn start(self, state: S, mut shutdown_signel: Receiver<()>) -> Result<()> {
        let state = Arc::new(state);

        let handlers = &self.handlers;
        let consumer: BaseConsumer = self.config.create()?;
        let topics: Vec<&str> = handlers.keys().cloned().collect();
        consumer.subscribe(&topics)?;

        info!("kakfa consumer started, topics={:?}", topics);

        loop {
            if let Some(result) = consumer.poll(Timeout::After(Duration::from_millis(500))) {
                let message = result?;
                let topic = message.topic().to_owned();

                if let Some(handler) = handlers.get(topic.as_str()) {
                    self.tasks.spawn(handler.handle(state.clone(), message));
                }
            }

            if shutdown_signel.try_recv().is_ok() {
                self.tasks.close();
                self.tasks.wait().await;

                info!("kakfa consumer stopped, topics={:?}", topics);
                return Ok(());
            }
        }
    }
}

impl<T: DeserializeOwned> From<BorrowedMessage<'_>> for Message<T> {
    fn from(message: BorrowedMessage<'_>) -> Message<T> {
        let key = message.key().map(|data| String::from_utf8_lossy(data).to_string());
        let value = message.payload().map(|data| String::from_utf8_lossy(data).to_string());

        let mut headers = HashMap::new();
        if let Some(kafka_headers) = message.headers() {
            for kafka_header in kafka_headers.iter() {
                headers.insert(
                    kafka_header.key.to_owned(),
                    kafka_header
                        .value
                        .map(|data| String::from_utf8_lossy(data).to_string())
                        .unwrap_or_default(),
                );
            }
        }

        let timestamp = match message.timestamp() {
            Timestamp::CreateTime(time) => DateTime::from_timestamp_millis(time),
            _ => None,
        };

        Message {
            key,
            payload: value.unwrap_or_default(),
            headers,
            timestamp,
            _marker: PhantomData,
        }
    }
}

async fn handle_message<H, S, M, Fut>(topic: &str, message: Message<M>, handler: H, state: Arc<S>)
where
    H: Fn(Arc<S>, Message<M>) -> Fut,
    Fut: Future<Output = Result<()>>,
    M: DeserializeOwned,
{
    let ref_id = message.headers.get("ref_id").map(|value| value.to_owned());
    log::start_action("kafka", ref_id, async {
        debug!(topic, "[message]");
        debug!(key = ?message.key, "[message]");
        debug!(
            timestamp = message
                .timestamp
                .map(|t| t.to_rfc3339_opts(SecondsFormat::Millis, true)),
            "[message]"
        );
        debug!(payload = message.payload, "[message]");
        for (key, value) in message.headers.iter() {
            debug!("[header] {}={}", key, value);
        }
        debug!(topic, key = message.key, "context");
        if let Some(timestamp) = message.timestamp {
            debug!("lag={:?}", duration(Utc::now(), timestamp));
        }
        handler(state, message).await
    })
    .await;
}
