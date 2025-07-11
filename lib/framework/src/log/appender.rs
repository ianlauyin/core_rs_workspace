use std::time::Duration;

use super::ActionLogAppender;
use super::ActionLogMessage;
use super::ActionResult;
use crate::json;

pub struct ConsoleAppender;

impl ActionLogAppender for ConsoleAppender {
    fn append(&self, action_log: ActionLogMessage) {
        let mut log = format!(
            "{} | {} | {} | id={}",
            action_log.date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            json::to_json_value(&action_log.result),
            action_log.action,
            action_log.id
        );

        if let Some(error_code) = action_log.error_code {
            log.push_str(&format!(" | error_code={error_code}"));
        }

        if let Some(error_message) = action_log.error_message {
            log.push_str(&format!(" | error_message={error_message}"));
        }

        if let Some(ref_id) = action_log.ref_id {
            log.push_str(&format!(" | ref_id={ref_id}"));
        }

        for (key, value) in action_log.context {
            log.push_str(&format!(" | {key}={value}"));
        }

        for (key, value) in action_log.stats {
            if key.ends_with("elapsed") {
                log.push_str(&format!(" | {key}={:?}", Duration::from_nanos(value as u64)));
            } else {
                log.push_str(&format!(" | {key}={value}"));
            }
        }

        println!("{log}");

        if action_log.result != ActionResult::Ok {
            if let Some(trace) = action_log.trace {
                eprintln!("{trace}");
            }
        }
    }
}
