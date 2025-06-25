use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use core_ng::kafka::consumer::Message;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionLogMessage {
    id: String,
    date: DateTime<Utc>,
    app: String,
    host: String,
    result: String,
    action: String,
    correlation_ids: Vec<String>,
    clients: Option<Vec<String>>,
    ref_ids: Option<Vec<String>>,
    error_code: Option<String>,
    error_message: Option<String>,
    elapsed: u64,
    context: HashMap<String, Vec<Option<String>>>,
    stats: HashMap<String, f64>,
    perf_stats: HashMap<String, PerformanceStatMessage>,
    trace_log: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceStatMessage {
    total_elapsed: u64,
    count: u32,
    read_entries: Option<u32>,
    write_entries: Option<u32>,
}

pub async fn action_log_message_handler(_state: Arc<AppState>, messages: Vec<Message<ActionLogMessage>>) -> Result<()> {
    for message in messages {
        println!("Received action log message: {:?}", message.payload());
    }
    Ok(())
}
