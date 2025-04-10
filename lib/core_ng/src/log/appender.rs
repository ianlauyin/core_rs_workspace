use std::fmt::Write;

use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
pub struct ActionLogMessage {
    pub id: String,
    pub date: DateTime<Utc>,
    pub action: String,
    pub result: &'static str,
    pub trace: Option<String>,
    pub elapsed: u128,
}

pub trait ActionLogAppender {
    fn append(&self, action_log: ActionLogMessage);
}

pub struct ConsoleAppender;

const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

impl ActionLogAppender for ConsoleAppender {
    fn append(&self, action_log: ActionLogMessage) {
        let mut log = String::new();
        write!(
            log,
            "{CYAN}{} | {} | elapsed={}ms | id={} | action={}{RESET}",
            action_log.date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            action_log.result,
            action_log.elapsed / 1_000_000,
            action_log.id,
            action_log.action
        )
        .unwrap();

        println!("{log}");

        if action_log.result != "Ok" {
            if let Some(trace) = action_log.trace {
                eprintln!("{RED}{trace}{RESET}");
            }
        }
    }
}
