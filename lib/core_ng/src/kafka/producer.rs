use std::fmt::Debug;

use anyhow::Result;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use serde::Serialize;

use super::topic::Topic;
use crate::json::to_json;

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
        let payload = to_json(message)?;
        let mut record = FutureRecord::<String, String>::to(topic.name).payload(&payload);
        if let Some(ref key) = key {
            record = record.key(key);
        }
        let result = self.producer.send(record, Timeout::Never).await;
        if let Err((err, _)) = result {
            return Err(err.into());
        }
        Ok(())
    }
}
