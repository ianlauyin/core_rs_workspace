use std::io;

use framework::exception;
use framework::exception::Severity;

macro_rules! log_event {
    (level = $level:ident, error_code = $error_code:ident, $($arg:tt)+) => {
        match $level {
            ::tracing::Level::TRACE => {},
            ::tracing::Level::DEBUG => {},
            ::tracing::Level::INFO => {},
            ::tracing::Level::WARN => {
                match $error_code {
                    Some(ref error_code) => ::tracing::warn!(error_code, $($arg)+),
                    None => ::tracing::warn!($($arg)+),
                }
            },
            ::tracing::Level::ERROR => {
                match $error_code {
                    Some(ref error_code) => ::tracing::error!(error_code, $($arg)+),
                    None => ::tracing::error!($($arg)+),
                }
            }
        }
    };
}

pub fn main() {
    framework::log::init();

    let level = tracing::Level::WARN;
    let error_code = Some("E001".to_string());

    log_event!(level = level, error_code = error_code, "hello");

    let error = io::Error::other("an error occurred");
    let e = exception!(
        severity = Severity::Warn,
        code = "Hello",
        message = "yes",
        source = error
    );
    println!("{e}");
}
