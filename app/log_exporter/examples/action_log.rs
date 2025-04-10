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
    log::start_action("some-action", async {
        let x = Arc::new(Mutex::new(1));
        let y = x.clone();

        task::spawn_action("some-action:task", async move {
            *y.lock().unwrap() = 2;
            warn!("y = {y:?}");
            Ok(())
        });

        warn!("after task, {x:?}");
        handle_request(false).await?;
        Ok(())
    })
    .await;
}

async fn handle_request(success: bool) -> Result<()> {
    let span = info_span!("http", some_thing = "some");
    span.in_scope(|| {
        info!(request_id = 123, "Processing request,");
    });

    async {
        info!("inside async block");
    }
    .await;

    let db_span = debug_span!("db");
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
