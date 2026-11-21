//! Production JSON logging always excludes WebSocket payload diagnostics.

use tracing::{Event, Metadata, Subscriber, subscriber::Interest};
use tracing_log::NormalizeEvent;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt};

pub(crate) fn subscriber(writer: BoxMakeWriter, directives: &str) -> impl Subscriber + Send + Sync {
    let verbosity = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(directives);
    Registry::default()
        .with(verbosity)
        .with(fmt::layer().json().with_ansi(false).with_writer(writer))
        .with(TransportSafety)
}

struct TransportSafety;

impl<S: Subscriber> Layer<S> for TransportSafety {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if safe_target(metadata.target()) {
            Interest::sometimes()
        } else {
            Interest::never()
        }
    }

    fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
        safe_target(metadata.target())
    }

    fn event_enabled(&self, event: &Event<'_>, _: Context<'_, S>) -> bool {
        match event.normalized_metadata() {
            Some(metadata) => safe_target(metadata.target()),
            None => safe_target(event.metadata().target()),
        }
    }
}

fn safe_target(target: &str) -> bool {
    !["tungstenite", "tokio_tungstenite"]
        .iter()
        .any(|namespace| {
            target == *namespace
                || target
                    .strip_prefix(namespace)
                    .is_some_and(|suffix| suffix.starts_with("::"))
        })
}

#[cfg(test)]
#[path = "_tests_/logging_tests.rs"]
mod logging_tests;
