use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMessage {
    id: String,
    date: DateTime<Utc>,
    app: String,
    received_time: DateTime<Utc>,
    result: String,
    action: String,
    error_code: Option<String>,
    error_message: Option<String>,
    elapsed: i64,
    context: HashMap<String, String>,
    stats: HashMap<String, f64>,
    info: HashMap<String, String>,
}
