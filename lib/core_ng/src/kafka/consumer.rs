use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use rdkafka::ClientConfig;
use rdkafka::Message as _;
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::util::Timeout;
use serde::de::DeserializeOwned;
use tokio::task::JoinHandle;
use tracing::info;

use crate::json::from_json;

pub struct Message<P: DeserializeOwned> {
    pub key: Option<String>,
    pub value: P,
}

pub trait MessageHandler<S, P: DeserializeOwned>: Send + Sync {
    fn handle(
        &self,
        state: Arc<S>,
        messages: Vec<Message<P>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync + 'static>>;
}

impl<F, Fut, S, P> MessageHandler<S, P> for F
where
    P: DeserializeOwned,
    F: Fn(Arc<S>, Vec<Message<P>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + Sync + 'static,
{
    fn handle(
        &self,
        state: Arc<S>,
        messages: Vec<Message<P>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync + 'static>> {
        Box::pin(self(state, messages))
    }
}

pub struct MessageConsumer<S> {
    config: ClientConfig,
    handlers: HashMap<&'static str, Arc<Box<dyn MessageHandler<S, String>>>>,
}

impl<S> MessageConsumer<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler<P, H, Fut>(&mut self, topic: &'static str, handler: H)
    where
        P: DeserializeOwned + Send + Sync + 'static,
        H: Fn(Arc<S>, Message<P>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        self.add_bulk_handler(topic, move |state: Arc<S>, messages: Vec<Message<P>>| {
            let handler = handler.clone();
            async move {
                let mut handles: Vec<JoinHandle<Result<()>>> = vec![];
                for message in messages {
                    let state = Arc::clone(&state);
                    handles.push(tokio::spawn(handler(state, message)));
                }
                for handle in handles {
                    handle.await??;
                }
                Ok(())
            }
        });
    }

    pub fn add_bulk_handler<P, H, Fut>(&mut self, topic: &'static str, handler: H)
    where
        P: DeserializeOwned + 'static,
        H: Fn(Arc<S>, Vec<Message<P>>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + Sync + 'static,
    {
        let handler = move |state: Arc<S>, messages: Vec<Message<String>>| {
            let messages = messages
                .into_iter()
                .map(|m| Message {
                    key: m.key,
                    value: from_json(&m.value).unwrap(),
                })
                .collect();
            handler.handle(state, messages)
        };

        self.handlers.insert(topic, Arc::new(Box::new(handler)));
    }

    pub async fn start(self, state: S) -> Result<()> {
        let state = Arc::new(state);

        let handlers = &self.handlers;
        let consumer: BaseConsumer = self.config.create()?;
        let topics: Vec<&str> = handlers.keys().cloned().collect();
        consumer.subscribe(&topics)?;

        info!("kakfa consumer started, topics={:?}", topics);

        loop {
            let messages = poll_messages(&consumer, Duration::from_secs(5), 1000)?;

            let mut handles: Vec<JoinHandle<Result<()>>> = vec![];

            for (topic, messages) in messages {
                let messages = messages
                    .into_iter()
                    .map(|m| Message {
                        key: m.key().map(|data| String::from_utf8_lossy(data).to_string()),
                        value: m
                            .payload()
                            .map(|data| String::from_utf8_lossy(data).to_string())
                            .unwrap(),
                    })
                    .collect();
                let handler = handlers.get(topic.as_str()).unwrap();
                let handler = Arc::clone(handler);
                let state = state.clone();
                handles.push(tokio::spawn(handler.handle(state, messages)));
            }

            for handle in handles {
                handle.await??;
            }

            consumer.commit_consumer_state(CommitMode::Async)?;
        }
    }
}

fn poll_messages(
    consumer: &'_ BaseConsumer,
    timeout: Duration,
    max_messages: usize,
) -> Result<HashMap<String, Vec<BorrowedMessage<'_>>>> {
    let mut messages: HashMap<String, Vec<BorrowedMessage<'_>>> = HashMap::new();
    let start_time = Instant::now();
    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout {
            break;
        }

        if messages.len() >= max_messages {
            break;
        }

        if let Some(result) = consumer.poll(Timeout::After(timeout.saturating_sub(elapsed))) {
            let message = result?;
            let topic = message.topic().to_owned();
            messages.entry(topic).or_default().push(message);
        }
    }
    Ok(messages)
}
