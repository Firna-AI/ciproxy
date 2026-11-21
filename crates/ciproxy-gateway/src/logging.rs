//! Mandatory hosted filtering for dependency logs that can expose wire secrets.

use std::env::VarError;
use std::fmt;

use tracing::{Event, Metadata, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::{LevelFilter, ParseError, Targets};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
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

fn protect(subscriber: Box<dyn Subscriber + Send + Sync>) -> Box<dyn Subscriber + Send + Sync> {
    Box::new(subscriber.with(FrameLogGuard))
}

/// A nonfatal notice from the pinned RUST_LOG selection rules.
#[derive(Debug)]
pub enum LogNotice {
    /// The entire invalid directive value is ignored.
    InvalidDirectives {
        /// The operator-supplied directive text.
        value: String,
        /// The original Targets parser diagnostic.
        error: ParseError,
    },
    /// An unreadable Unicode value uses the default INFO selection.
    NonUnicode(VarError),
}

impl fmt::Display for LogNotice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDirectives { value, error } => {
                write!(formatter, "Ignoring `RUST_LOG={value:?}`: {error}")
            }
            Self::NonUnicode(error) => write!(formatter, "Ignoring `RUST_LOG`: {error}"),
        }
    }
}

/// A guarded subscriber and any configuration notice for the CLI to print.
pub struct HostedSubscriber {
    /// Subscriber ready for global installation.
    pub subscriber: Box<dyn Subscriber + Send + Sync>,
    /// Nonfatal notice that remains visible even when log output is disabled.
    pub notice: Option<LogNotice>,
}

fn default_targets() -> Targets {
    Targets::new().with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL)
}

fn selection(rust_log: Result<String, VarError>) -> (Targets, Option<LogNotice>) {
    match rust_log {
        Ok(value) => match value.parse() {
            Ok(targets) => (targets, None),
            Err(error) => (
                Targets::default(),
                Some(LogNotice::InvalidDirectives { value, error }),
            ),
        },
        Err(VarError::NotPresent) => (default_targets(), None),
        Err(error) => (default_targets(), Some(LogNotice::NonUnicode(error))),
    }
}

/// Construct hosted logging with the pinned Targets grammar and mandatory frame guard.
///
/// Pass the result of reading RUST_LOG and the selected output writer.
/// The caller prints any notice to stderr before installing the subscriber.
pub fn subscriber(rust_log: Result<String, VarError>, writer: BoxMakeWriter) -> HostedSubscriber {
    let (targets, notice) = selection(rust_log);
    let formatted = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_writer(writer)
        .finish()
        .with(targets);
    HostedSubscriber {
        subscriber: protect(Box::new(formatted)),
        notice,
    }
}
