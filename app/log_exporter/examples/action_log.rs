use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use anyhow::anyhow;
use core_ng::log::appender::ConsoleAppender;
use core_ng::log::{self};
use core_ng::task;
use tokio::task::yield_now;
use tracing::Instrument;
use tracing::debug;
use tracing::debug_span;
use tracing::error;
use tracing::field;
use tracing::info;
use tracing::info_span;
use tracing::warn;

#[tokio::main]
async fn main() {
    log::init(ConsoleAppender);

    test_action().await;

    task::shutdown().await;
}

async fn test_action() {
    log::start_action("some-action".to_string(), None, async {
        let x = Arc::new(Mutex::new(1));
        let y = x.clone();

        debug!(key = "value1", key2 = "value2", "context");

        task::spawn_action("some-task", async move {
            *y.lock().unwrap() = 2;
            warn!("y = {y:?}");
            Ok(())
        });

        debug!(key3 = "value3", "context");

        warn!("after task, {}", x.lock().unwrap());
        handle_request(false).await?;
        Ok(())
    })
    .await;
}

async fn handle_request(success: bool) -> Result<()> {
    let span = info_span!("http", elapsed = field::Empty);
    async {
        info!(request_id = 123, "Processing request,");
    }
    .instrument(span)
    .await;

    async {
        info!("inside async block");
    }
    .await;

    let db_span = debug_span!("db", elapsed = field::Empty);
    async {
        debug!(sql = "select 1", "run db query,");
    }
    .instrument(db_span)
    .await;

    yield_now().await;

    other_method().await;

    if success {
        info!(status = "success", "Request completed successfully,");
        Ok(())
    } else {
        warn!(status = "failure", "Something went wrong,");
        error!(reason = "database_error", "Could not connect to database,");
        Err(anyhow!("key length must be 16 characters, got {:?}", "key"))
    }
}

async fn other_method() {
    info!("other_method");
}
