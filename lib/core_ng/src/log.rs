use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Result;
pub use appender::ConsoleAppender;
use chrono::DateTime;
use chrono::Utc;
use layer::ActionLogLayer;
use serde::Serialize;
use tokio::task_local;
use tracing::Instrument;
use tracing::error;
use tracing::info_span;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

mod appender;
mod layer;

pub trait ActionLogAppender {
    fn append(&self, action_log: ActionLogMessage);
}

pub fn init<T>(appender: T)
where
    T: ActionLogAppender + Send + Sync + 'static,
{
    tracing_subscriber::registry()
        .with(ActionLogLayer { appender })
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(LevelFilter::INFO),
        )
        .init();
}

task_local! {
    pub(crate) static CURRENT_ACTION_ID: String
}

pub async fn start_action<T>(action: &str, ref_id: Option<String>, task: T)
where
    T: Future<Output = Result<()>>,
{
    let action_id = Uuid::now_v7().to_string();
    let action_span = info_span!("action", action, action_id, ref_id);
    CURRENT_ACTION_ID
        .scope(
            action_id,
            async {
                let result = task.await;
                if let Err(e) = result {
                    error!("{}\n{}", e, e.backtrace());
                }
            }
            .instrument(action_span),
        )
        .await;
}

#[derive(Serialize, Debug)]
pub struct ActionLogMessage {
    pub id: String,
    pub date: DateTime<Utc>,
    pub action: String,
    pub result: ActionResult,
    pub ref_id: Option<String>,
    pub context: HashMap<String, String>,
    pub trace: Option<String>,
    pub elapsed: u128,
}

#[derive(PartialEq, Serialize, Debug)]
pub enum ActionResult {
    Ok,
    Warn,
    Error,
}

impl ActionResult {
    fn level(&self) -> u32 {
        match self {
            ActionResult::Ok => 0,
            ActionResult::Warn => 1,
            ActionResult::Error => 2,
        }
    }
}
