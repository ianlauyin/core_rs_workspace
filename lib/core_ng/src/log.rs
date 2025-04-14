use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Write;
use std::time::Instant;

use anyhow::Result;
use appender::ActionLogAppender;
use appender::ActionLogMessage;
use chrono::DateTime;
use chrono::Utc;
use tokio::task_local;
use tracing::Event;
use tracing::Instrument;
use tracing::Level;
use tracing::Subscriber;
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
                    warn!("{}\n{}", e, e.backtrace());
                }
            }
            .instrument(action_span),
        )
        .await;
}

struct ActionLog {
    id: String,
    action: String,
    date: DateTime<Utc>,
    start_time: Instant,
    result: ActionResult,
    ref_id: Option<String>,
    context: HashMap<String, String>,
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
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let span = context.span(id).unwrap();
        let mut extensions = span.extensions_mut();

        if span.name() == "action" {
            let mut action_visitor = ActionVisitor::new();
            attrs.record(&mut action_visitor);
            if let Some(mut action_log) = action_visitor.action_log() {
                if extensions.get_mut::<ActionLog>().is_none() {
                    action_log.logs.push(format!(
                        r#"=== action begin ===
action={}
id={}
date={}"#,
                        action_log.action,
                        action_log.id,
                        action_log.date.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                    ));

                    if let Some(ref ref_id) = action_log.ref_id {
                        action_log.logs.push(format!("ref_id={}", ref_id));
                    }

                    extensions.insert(action_log);
                }
            }
        } else if span.fields().field("elapsed").is_some() {
            extensions.insert(PerformanceSpan {
                start_time: Instant::now(),
            });
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        if let Some(action_log) = span.extensions_mut().remove::<ActionLog>() {
            let action_log_message = close_action(action_log);
            self.appender.append(action_log_message);
        } else if let Some(performance_span) = span.extensions_mut().remove::<PerformanceSpan>() {
            if let Some(action_span) = action_span(&span) {
                if let Some(action_log) = action_span.extensions_mut().get_mut::<ActionLog>() {
                    action_log.logs.push(format!(
                        "{}, elapsed={:?}",
                        span.name(),
                        performance_span.start_time.elapsed()
                    ));
                }
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let mut action_span: Option<SpanRef<'_, S>> = None;
        let mut spans = String::new();

        if let Some(scope) = context.event_scope(event) {
            for span in scope.from_root() {
                if span.name() == "action" && span.extensions().get::<ActionLog>().is_some() {
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

                // hanldle "context" event
                let mut context_log_visitor = ContextLogVisitor { context: None };
                event.record(&mut context_log_visitor);
                if let Some(context) = context_log_visitor.context {
                    action_log.context.extend(context);
                }
            }
        }
    }
}

struct PerformanceSpan {
    start_time: Instant,
}

struct ActionVisitor {
    action: Option<String>,
    action_id: Option<String>,
    ref_id: Option<String>,
}

impl ActionVisitor {
    fn new() -> Self {
        Self {
            action: None,
            action_id: None,
            ref_id: None,
        }
    }

    fn action_log(self) -> Option<ActionLog> {
        if let (Some(action), Some(action_id)) = (self.action, self.action_id) {
            Some(ActionLog {
                id: action_id,
                action,
                date: Utc::now(),
                start_time: Instant::now(),
                result: ActionResult::Ok,
                ref_id: self.ref_id,
                context: HashMap::new(),
                logs: Vec::new(),
            })
        } else {
            None
        }
    }
}

impl Visit for ActionVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "action" => self.action = Some(value.to_string()),
            "action_id" => self.action_id = Some(value.to_string()),
            "ref_id" => self.ref_id = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn Debug) {}
}

fn action_span<'b, S>(current_span: &SpanRef<'b, S>) -> Option<SpanRef<'b, S>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    current_span
        .scope()
        .find(|parent_span| parent_span.name() == "action" && parent_span.extensions().get::<ActionLog>().is_some())
}

fn close_action(mut action_log: ActionLog) -> ActionLogMessage {
    let elapsed = action_log.start_time.elapsed();
    let mut trace = None;
    if action_log.result.level() > ActionResult::Ok.level() {
        action_log.logs.push(format!(
            r#"elapsed={:?}
=== action end ===
"#,
            elapsed
        ));
        trace = Some(action_log.logs.join("\n"));
    }

    ActionLogMessage {
        id: action_log.id,
        date: action_log.date,
        action: action_log.action,
        result: action_log.result.as_str(),
        ref_id: action_log.ref_id,
        context: action_log.context,
        trace,
        elapsed: elapsed.as_nanos(),
    }
}

struct LogVisitor<'a>(&'a mut String);

impl Visit for LogVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.push_str(format!("{}={} ", field.name(), value).as_str());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.0.push_str(format!("{:?} ", value).as_str());
        } else {
            self.0.push_str(format!("{}={:?} ", field.name(), value).as_str());
        }
    }
}

struct ContextLogVisitor {
    context: Option<HashMap<String, String>>,
}

impl Visit for ContextLogVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if let Some(ref mut context) = self.context {
            context.insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.context = if format!("{value:?}") == "context" {
                Some(HashMap::new())
            } else {
                None
            };
        } else if let Some(ref mut context) = self.context {
            context.insert(field.name().to_string(), format!("{:?}", value));
        }
    }
}
