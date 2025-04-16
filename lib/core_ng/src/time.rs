use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use chrono::DateTime;
use chrono::Utc;

pub fn current_time_millis() -> u64 {
    let now = SystemTime::now();
    if let Ok(duration) = now.duration_since(UNIX_EPOCH) {
        duration.as_millis() as u64
    } else {
        0
    }
}

pub fn duration(from: DateTime<Utc>, to: DateTime<Utc>) -> Duration {
    let elapsed = to - from;
    Duration::from_millis(elapsed.num_milliseconds() as u64)
}
