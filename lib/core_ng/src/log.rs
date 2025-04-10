use std::fmt::Debug;
use std::fmt::Write;
use std::time::Instant;

use anyhow::Result;
use appender::ActionLogAppender;
use appender::ActionLogMessage;
use chrono::DateTime;
use chrono::Utc;
use tracing::Event;
use tracing::Instrument;
use tracing::Level;
use tracing::Subscriber;
use tracing::debug;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::info_span;
use tracing::level_filters::LevelFilter;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::warn;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::registry::SpanRef;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

pub mod appender;

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

pub async fn start_action<T>(action: &str, task: T)
where
    T: Future<Output = Result<()>>,
{
    let action_id = Uuid::now_v7().to_string();
    let action_span = info_span!("action", action, action_id);
    async move {
        debug!("=== action begin ===");
        debug!("action={}", action);
        debug!("id={}", action_id);
        debug!(
            "date={}",
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
        );
        let result = task.await;
        if let Err(e) = result {
            warn!("{}\n{}", e, e.backtrace());
        }
        debug!("=== action end ===");
    }
    .instrument(action_span)
    .await
}

struct ActionLog {
    id: String,
    action: String,
    date: DateTime<Utc>,
    start_time: Instant,
    result: ActionResult,
    logs: Vec<String>,
}

#[derive(Debug)]
enum ActionResult {
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

    fn as_str(&self) -> &'static str {
        match self {
            ActionResult::Ok => "Ok",
            ActionResult::Warn => "Warn",
            ActionResult::Error => "Error",
        }
    }
}

struct ActionLogLayer<T>
where
    T: ActionLogAppender,
{
    appender: T,
}

impl<T, S> Layer<S> for ActionLogLayer<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    T: ActionLogAppender + 'static,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();

        if span.metadata().name() == "action" {
            let mut action_visitor = ActionVisitor::new();
            attrs.record(&mut action_visitor);
            if let Some(action_log) = action_visitor.action_log() {
                if extensions.get_mut::<ActionLog>().is_none() {
                    extensions.insert(action_log);
                }
            }
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        if let Some(action_log) = span.extensions().get::<ActionLog>() {
            let action_log_message = close_action(action_log);
            self.appender.append(action_log_message);
        }
    }

    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let mut action_span: Option<SpanRef<'_, S>> = None;
        let mut spans = String::new();

        if let Some(scope) = context.event_scope(event) {
            for span in scope.from_root() {
                if span.extensions().get::<ActionLog>().is_some() {
                    action_span = Some(span);
                } else {
                    if !spans.is_empty() {
                        spans.push(':');
                    }
                    spans.push_str(span.metadata().name());
                }
            }
        }

        if let Some(span) = action_span {
            if let Some(action_log) = span.extensions_mut().get_mut::<ActionLog>() {
                let elapsed = action_log.start_time.elapsed();
                let total_seconds = elapsed.as_secs();
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;
                let nanos = elapsed.subsec_nanos();

                let mut log_string = String::new();
                write!(log_string, "{:02}:{:02}.{:09} ", minutes, seconds, nanos).unwrap();

                let metadata = event.metadata();
                let level = metadata.level();
                if level <= &Level::INFO {
                    write!(log_string, "{} ", level).unwrap();

                    if level == &Level::ERROR && action_log.result.level() < ActionResult::Error.level() {
                        action_log.result = ActionResult::Error;
                    } else if level == &Level::WARN && action_log.result.level() < ActionResult::Warn.level() {
                        action_log.result = ActionResult::Warn;
                    }
                }

                if !spans.is_empty() {
                    write!(log_string, "{} ", spans).unwrap();
                }

                write!(log_string, "{}:{} ", metadata.target(), metadata.line().unwrap_or(0)).unwrap();

                let mut visitor = LogVisitor(&mut log_string);
                event.record(&mut visitor);

                action_log.logs.push(log_string);
            }
        }
    }
}

struct LogVisitor<'a>(&'a mut String);

impl Visit for LogVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.0.push_str(format!("{:?} ", value).as_str());
        } else {
            self.0.push_str(format!("{}={:?} ", field.name(), value).as_str());
        }
    }
}

struct ActionVisitor {
    action: Option<String>,
    action_id: Option<String>,
    // ref_action_id: Option<u64>,
}

impl ActionVisitor {
    fn new() -> Self {
        Self {
            action: None,
            action_id: None,
            // ref_action_id: None,
        }
    }

    fn action_log(&self) -> Option<ActionLog> {
        if let (Some(action), Some(action_id)) = (self.action.as_ref(), self.action_id.as_ref()) {
            Some(ActionLog {
                id: action_id.to_string(),
                action: action.to_string(),
                date: Utc::now(),
                start_time: Instant::now(),
                result: ActionResult::Ok,
                logs: Vec::new(),
            })
        } else {
            None
        }
    }
}

impl Visit for ActionVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "action" {
            self.action = Some(value.to_string());
        } else if field.name() == "action_id" {
            self.action_id = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn Debug) {}
}

// fn strip_quotes(value: String) -> String {
//     if value.starts_with('"') && value.ends_with('"') {
//         value[1..value.len() - 1].to_string()
//     } else {
//         value
//     }
// }

fn close_action(action_log: &ActionLog) -> ActionLogMessage {
    let trace = if action_log.result.level() > ActionResult::Ok.level() {
        Some(action_log.logs.join("\n"))
    } else {
        None
    };

    ActionLogMessage {
        id: action_log.id.to_string(),
        date: action_log.date,
        action: action_log.action.to_string(),
        result: action_log.result.as_str(),
        trace,
        elapsed: action_log.start_time.elapsed().as_nanos(),
    }
}
