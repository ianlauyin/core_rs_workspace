use std::fmt::Debug;

use anyhow::Result;
pub use appender::ConsoleAppender;
use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
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

task_local! {
    static CURRENT_ACTION_ID: String
}

pub fn init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(LevelFilter::INFO),
        )
        .init();
}

pub fn init_with_action<T>(appender: T)
where
    T: ActionLogAppender + Send + Sync + 'static,
{
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(LevelFilter::INFO),
        )
        .with(ActionLogLayer { appender })
        .init();
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

pub fn current_action_id() -> Option<String> {
    CURRENT_ACTION_ID
        .try_with(|current_action_id| Some(current_action_id.clone()))
        .unwrap_or(None)
}

#[derive(Serialize, Debug)]
pub struct ActionLogMessage {
    pub id: String,
    pub date: DateTime<Utc>,
    pub action: String,
    pub result: ActionResult,
    pub ref_id: Option<String>,
    pub context: IndexMap<&'static str, String>,
    pub stats: IndexMap<String, u128>,
    pub trace: Option<String>,
}

#[derive(PartialEq, Serialize, Debug)]
pub enum ActionResult {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "ERROR")]
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
