use std::collections::HashMap;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use framework::exception::Exception;
use framework::kafka::consumer::Message;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;

// action log message schema from java core-ng framework
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionLogMessage {
    id: String,
    date: DateTime<Utc>,
    app: String,
    host: String,
    result: String,
    action: String,
    correlation_ids: Option<Vec<String>>,
    clients: Option<Vec<String>>,
    ref_ids: Option<Vec<String>>,
    error_code: Option<String>,
    error_message: Option<String>,
    elapsed: i64,
    context: HashMap<String, Vec<Option<String>>>,
    stats: HashMap<String, f64>,
    perf_stats: HashMap<String, PerformanceStatMessage>,
    trace_log: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceStatMessage {
    total_elapsed: i64,
    count: i64,
    read_entries: Option<i64>,
    write_entries: Option<i64>,
}

pub async fn action_log_message_handler(
    _state: Arc<AppState>,
    _messages: Vec<Message<ActionLogMessage>>,
) -> Result<(), Exception> {
    Ok(())
}
