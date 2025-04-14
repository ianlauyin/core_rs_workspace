use std::future::Future;
use std::sync::LazyLock;

use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;
use tracing::Instrument;
use tracing::Span;
use tracing::info;

use crate::log;
use crate::log::CURRENT_ACTION_ID;

static TASK_TRACKER: LazyLock<TaskTracker> = LazyLock::new(TaskTracker::new);

pub fn spawn_action<T>(action: &str, task: T)
where
    T: Future<Output = Result<()>> + Send + 'static,
{
    let action = action.to_string();
    let ref_id = CURRENT_ACTION_ID
        .try_with(|current_action_id| Some(current_action_id.clone()))
        .unwrap_or(None);
    TASK_TRACKER.spawn(async move { log::start_action(action.as_str(), ref_id, task).await });
}

pub fn spawn_task<T>(task: T) -> JoinHandle<Result<()>>
where
    T: Future<Output = Result<()>> + Send + 'static,
{
    let span = Span::current();
    TASK_TRACKER.spawn(task.instrument(span))
}

pub async fn shutdown() {
    info!("waiting for {} task(s) to finish", TASK_TRACKER.len());
    TASK_TRACKER.close();
    TASK_TRACKER.wait().await;
    info!("tasks finished");
}
