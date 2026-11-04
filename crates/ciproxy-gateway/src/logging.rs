//! Mandatory hosted filtering for dependency logs that can expose wire secrets.

use tracing::{Event, Metadata, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};

fn permitted(target: &str) -> bool {
    !["tungstenite", "tokio_tungstenite"].iter().any(|prefix| {
        target == *prefix
            || target
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with("::"))
    })
}

struct FrameLogGuard;

impl<S: Subscriber> Layer<S> for FrameLogGuard {
    fn enabled(&self, metadata: &Metadata<'_>, _context: Context<'_, S>) -> bool {
        permitted(metadata.target())
    }

    fn event_enabled(&self, event: &Event<'_>, _context: Context<'_, S>) -> bool {
        match event.normalized_metadata() {
            Some(metadata) => permitted(metadata.target()),
            None => permitted(event.metadata().target()),
        }
    }
}

/// Wrap a hosted subscriber with a mandatory frame-log veto.
///
/// Apply this after every verbosity filter. Native targets and normalized
/// log-to-tracing targets are checked independently of user directives.
pub fn protect(subscriber: Box<dyn Subscriber + Send + Sync>) -> Box<dyn Subscriber + Send + Sync> {
    Box::new(subscriber.with(FrameLogGuard))
}
