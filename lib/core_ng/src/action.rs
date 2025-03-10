use std::cell::RefCell;
use std::fmt::Write;

use chrono::Utc;
use tokio::task_local;
use tracing::Event;
use tracing::Instrument;
use tracing::Level;
use tracing::Subscriber;
use tracing::field::Visit;
use tracing::info_span;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

// pub fn send() {
//     let (tx, mut rx) = mpsc::unbounded_channel();

//     // Spawn a task to handle log flushing
//     tokio::spawn(async move {
//         while let Some(logs) = rx.recv().await {
//             destination.flush(logs);
//         }
//     });
// }

task_local! {
    static ACTION_LOG: RefCell<ActionLog>;
}

#[allow(unused)]
#[derive(Debug, Default)]
struct ActionLog {
    action: String,
    logs: Vec<String>,
}

#[allow(unused)]
impl ActionLog {
    pub async fn start_action(action: impl AsyncFn()) {
        ACTION_LOG
            .scope(RefCell::new(ActionLog::default()), async {
                let span = info_span!("action", action_id = 123);

                async move {
                    action().await;
                }
                .instrument(span)
                .await;

                ACTION_LOG.with(|log| {
                    for log in log.borrow().logs.iter() {
                        println!("{log}")
                    }
                });
            })
            .await
    }
}

struct SpanField {
    value: String,
}

#[derive(Default)]
pub struct ActionLogLayer;

impl<S> Layer<S> for ActionLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if extensions.get_mut::<SpanField>().is_none() {
            let mut fields = SpanField {
                value: String::new(),
            };

            let mut log_string = String::new();
            let mut visitor = LogVisitor(&mut log_string);
            attrs.record(&mut visitor);
            fields.value = log_string;

            extensions.insert(fields);
        }
    }

    fn on_record(&self, id: &Id, _values: &Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        span.extensions_mut();
    }

    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let level = *event.metadata().level();

        let mut log_string = String::new();
        write!(log_string, "{} ", Utc::now()).unwrap();
        let mut visitor = LogVisitor(&mut log_string);
        event.record(&mut visitor);

        if let Some(scope) = context.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                let span_field = extensions.get::<SpanField>();
                write!(
                    log_string,
                    "span={} {}",
                    span.metadata().name(),
                    span_field.unwrap().value
                )
                .unwrap();
            }
        }

        ACTION_LOG
            .try_with(|logs| {
                // println!("process event");
                let mut logs = logs.borrow_mut();
                logs.logs.push(log_string);

                // Check if this is a warning or error
                if level <= Level::WARN {
                    // logs.has_warnings_or_errors = true;
                }
            })
            .unwrap();
    }
}

// Simple visitor to convert event fields to a string
struct LogVisitor<'a>(&'a mut String);

impl Visit for LogVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // println!("field => {}={:?}", field.name(), value);
        self.0
            .push_str(format!("{}: {:?} ", field.name(), value).as_str());
    }
}

#[cfg(test)]
mod tests {
    use tokio::task::yield_now;
    use tracing::error;
    use tracing::info;
    use tracing::info_span;
    use tracing::warn;
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::action::ActionLog;
    use crate::action::ActionLogLayer;

    #[tokio::test]
    async fn log() {
        let action_layer = ActionLogLayer {};

        tracing_subscriber::registry()
            .with(action_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_filter(tracing::level_filters::LevelFilter::INFO),
            )
            .init();

        // Simulate some requests
        handle_request(true).await; // This should not flush logs (no warnings/errors)
        handle_request(false).await; // This should flush logs (has warnings/errors)

        println!("Application completed");
    }

    async fn handle_request(success: bool) {
        ActionLog::start_action(async || {
            let span = info_span!("test", some_thing = "some");
            let _guard = span.enter();
            info!(request_id = 123, "Processing request");

            async {
                info!("inside async block");
            }
            .await;

            yield_now().await;

            if success {
                info!(status = "success", "Request completed successfully");
            } else {
                warn!(status = "failure", "Something went wrong");
                error!(reason = "database_error", "Could not connect to database");
            }
        })
        .await;
    }
}
