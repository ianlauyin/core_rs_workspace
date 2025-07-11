use std::fmt::Debug;

use chrono::Utc;
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

use super::topic::Topic;
use crate::env;
use crate::exception::Exception;
use crate::json::to_json;
use crate::log::current_action_id;

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

    pub async fn send<T>(&self, topic: &Topic<T>, key: Option<String>, message: &T) -> Result<(), Exception>
    where
        T: Serialize + Debug,
    {
        let span = debug_span!("message_producer", topic = topic.name, key);
        async {
            let payload = to_json(message)?;

            let mut record = FutureRecord::<String, String>::to(topic.name)
                .timestamp(Utc::now().timestamp_millis())
                .payload(&payload);

            if let Some(ref key) = key {
                record = record.key(key);
            }

            let mut headers = insert_header(OwnedHeaders::new(), "client", env::APP_NAME);
            if let Some(ref ref_id) = current_action_id() {
                headers = insert_header(headers, "ref_id", ref_id);
            }
            record = record.headers(headers);

            debug!(topic = topic.name, key, payload, "send");
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

fn insert_header(headers: OwnedHeaders, key: &str, value: &str) -> OwnedHeaders {
    headers.insert(Header {
        key,
        value: Some(value.as_bytes()),
    })
}
