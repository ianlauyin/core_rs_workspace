use std::collections::HashMap;
use std::fmt::Write;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
pub struct ActionLogMessage {
    pub id: String,
    pub date: DateTime<Utc>,
    pub action: String,
    pub result: &'static str,
    pub ref_id: Option<String>,
    pub context: HashMap<String, String>,
    pub trace: Option<String>,
    pub elapsed: u128,
}

pub trait ActionLogAppender {
    fn append(&self, action_log: ActionLogMessage);
}

pub struct ConsoleAppender;

impl ActionLogAppender for ConsoleAppender {
    fn append(&self, action_log: ActionLogMessage) {
        let mut log = String::new();
        write!(
            log,
            "{} | {} | elapsed={:?} | id={} | action={}",
            action_log.date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            action_log.result,
            Duration::from_nanos(action_log.elapsed as u64),
            action_log.id,
            action_log.action
        )
        .unwrap();

        if let Some(ref_id) = action_log.ref_id {
            write!(log, " | ref_id={ref_id}").unwrap();
        }

        for (key, value) in action_log.context {
            write!(log, " | {key}={value}").unwrap();
        }

        println!("{log}");

        if action_log.result != "Ok" {
            if let Some(trace) = action_log.trace {
                eprintln!("{trace}");
            }
        }
    }
}
