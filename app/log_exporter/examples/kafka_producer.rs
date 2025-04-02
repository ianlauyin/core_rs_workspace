use core_ng::json::to_json;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
struct TestMessage {
    name: String,
}

#[tokio::main]
pub async fn main() {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", "dev.internal:9092")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    for i in 1..100 {
        producer
            .send(
                FutureRecord::to("test_single")
                    .key("key")
                    .payload(to_json(&TestMessage { name: format!("{}", i) }).unwrap().as_str()),
                Timeout::Never,
            )
            .await
            .unwrap();
    }
}
