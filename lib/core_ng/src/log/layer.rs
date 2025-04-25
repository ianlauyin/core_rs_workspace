use std::fmt::Debug;
use std::fmt::Write;
use std::thread;
use std::time::Instant;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use tracing::Event;
use tracing::Level;
use tracing::Subscriber;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::registry::SpanRef;

use super::ActionLogAppender;
use super::ActionLogMessage;
use super::ActionResult;

pub(super) struct ActionLogLayer<T>
where
    T: ActionLogAppender,
{
    pub(super) appender: T,
}

struct ActionLog {
    id: String,
    action: String,
    date: DateTime<Utc>,
    start_time: Instant,
    result: ActionResult,
    ref_id: Option<String>,
    context: IndexMap<&'static str, String>,
    stats: IndexMap<String, u128>,
    logs: Vec<String>,
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
type={}
id={}
date={}
thread={:?}"#,
                        action_log.action,
                        action_log.id,
                        action_log.date.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                        thread::current().id()
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
                    let elapsed = performance_span.start_time.elapsed();
                    action_log.logs.push(format!("{}, elapsed={:?}", span.name(), elapsed));

                    let value = action_log.stats.entry(format!("{}_elapsed", span.name())).or_default();
                    *value += elapsed.as_nanos();

                    let value = action_log.stats.entry(format!("{}_count", span.name())).or_default();
                    *value += 1;
                }
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        if event.metadata().level() == &Level::TRACE {
            return;
        }

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
                let mut context_visitor = ContextVisitor { context: None };
                event.record(&mut context_visitor);
                if let Some(context) = context_visitor.context {
                    action_log.context.extend(context);
                }

                // hanldle "stats" event
                let mut stats_visitor = StatsVisitor { stats: None };
                event.record(&mut stats_visitor);
                if let Some(stats) = stats_visitor.stats {
                    for (key, value) in stats {
                        let stats_value = action_log.stats.entry(key.to_owned()).or_default();
                        *stats_value += value;
                    }
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
                context: IndexMap::new(),
                stats: IndexMap::new(),
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
    action_log.stats.insert("elapsed".to_owned(), elapsed.as_nanos());
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
        result: action_log.result,
        ref_id: action_log.ref_id,
        context: action_log.context,
        stats: action_log.stats,
        trace,
    }
}

struct LogVisitor<'a>(&'a mut String);

impl Visit for LogVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.push_str(&format!("{}={} ", field.name(), value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?} ", value));
        } else {
            self.0.push_str(&format!("{}={:?} ", field.name(), value));
        }
    }
}

struct ContextVisitor {
    context: Option<IndexMap<&'static str, String>>,
}

impl Visit for ContextVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if let Some(ref mut context) = self.context {
            context.insert(field.name(), value.to_owned());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.context = if format!("{value:?}") == "context" {
                Some(IndexMap::new())
            } else {
                None
            };
        } else if let Some(ref mut context) = self.context {
            context.insert(field.name(), format!("{:?}", value));
        }
    }
}

struct StatsVisitor {
    stats: Option<IndexMap<&'static str, u128>>,
}

impl Visit for StatsVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_u128(field, value as u128);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_u128(field, value as u128);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_u128(field, value as u128);
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_u128(field, value as u128);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        if let Some(ref mut stats) = self.stats {
            stats.insert(field.name(), value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.stats = if format!("{value:?}") == "stats" {
                Some(IndexMap::new())
            } else {
                None
            };
        }
    }
}
