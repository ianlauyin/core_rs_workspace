use std::fmt::Debug;

use anyhow::Result;
use rdkafka::ClientConfig;
use rdkafka::message::Header;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use serde::Serialize;
use tracing::Instrument;
use tracing::debug;
use tracing::debug_span;
use tracing::field;

use super::topic::Topic;
use crate::json::to_json;
use crate::log::CURRENT_ACTION_ID;

pub struct ProducerConfig {
    pub bootstrap_servers: &'static str,
}

pub struct Producer {
    producer: FutureProducer,
}

impl Producer {
    pub fn new(config: ProducerConfig) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", config.bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .set("compression.codec", "zstd")
            .create()
            .expect("Producer creation error");
        Self { producer }
    }

    pub async fn send<T>(&self, topic: &Topic<T>, key: Option<String>, message: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let span = debug_span!("kafka", elapsed = field::Empty);
        async {
            let payload = to_json(message)?;
            let mut record = FutureRecord::<String, String>::to(topic.name).payload(&payload);
            if let Some(ref key) = key {
                record = record.key(key);
            }
            let ref_id = CURRENT_ACTION_ID
                .try_with(|current_action_id| Some(current_action_id.clone()))
                .unwrap_or(None);
            if let Some(ref_id) = ref_id {
                let headers = OwnedHeaders::new();
                let headers = headers.insert(Header {
                    key: "ref_id",
                    value: Some(ref_id.as_bytes()),
                });
                record = record.headers(headers);
            }
            debug!(key, payload, "send");
            let result = self.producer.send(record, Timeout::Never).await;
            if let Err((err, _)) = result {
                return Err(err.into());
            }
            Ok(())
        }
        .instrument(span)
        .await
    }
}
