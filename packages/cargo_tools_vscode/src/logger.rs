use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use crate::vs_code_api::{log_debug, log_error, log_info, log_trace, log_warn};

pub struct VSCodeLogger;

struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_owned();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

impl<S: Subscriber> Layer<S> for VSCodeLogger {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        let msg = &visitor.0;
        match *event.metadata().level() {
            Level::ERROR => log_error(msg),
            Level::WARN => log_warn(msg),
            Level::INFO => log_info(msg),
            Level::DEBUG => log_debug(msg),
            Level::TRACE => log_trace(msg),
        }
    }
}
