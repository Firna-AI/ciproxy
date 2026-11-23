//! A log text for an error that keeps the safe part of its cause.
//!
//! `Error::Internal` displays only "internal error", so a log of that text
//! hides why a command failed. [`diagnostic`] adds the context frames,
//! which are fixed texts of the agent, and each cause whose text holds no
//! data: an operating system error, a JSON position, an agent error, and
//! the kind of an HTTP failure. An HTTP error displays its URL, and an
//! image URL is a signed link, so the text never holds a URL. Any other
//! cause is named only as "another cause".

use std::error::Error as StdError;

use crate::error::Error;

/// The display text, then for an internal error its contexts, from the
/// outer one to the inner one, its causes, and where it was made.
pub(crate) fn diagnostic(error: &Error) -> String {
    let Error::Internal(internal) = error else {
        return error.to_string();
    };
    let mut parts: Vec<String> = internal
        .context_frames()
        .iter()
        .rev()
        .map(|frame| frame.message().to_owned())
        .collect();
    parts.extend(internal.source_chain().map(cause));
    parts.dedup();
    format!(
        "{error}: {} (at {})",
        parts.join(": "),
        internal.caller_at()
    )
}

fn cause(source: &(dyn StdError + 'static)) -> String {
    if let Some(io) = source.downcast_ref::<std::io::Error>() {
        return match io.raw_os_error() {
            Some(_) => io.to_string(),
            None => format!("{:?}", io.kind()),
        };
    }
    if let Some(json) = source.downcast_ref::<serde_json::Error>() {
        return format!("JSON {json}");
    }
    if let Some(agent) = source.downcast_ref::<Error>() {
        return match agent {
            Error::Internal(_) => "another cause".to_owned(),
            agent => agent.to_string(),
        };
    }
    if let Some(http) = source.downcast_ref::<reqwest::Error>() {
        return http_kind(http);
    }
    "another cause".to_owned()
}

fn http_kind(http: &reqwest::Error) -> String {
    if let Some(status) = http.status() {
        format!("HTTP status {}", status.as_u16())
    } else if http.is_timeout() {
        "the HTTP request timed out".to_owned()
    } else if http.is_connect() {
        "the HTTP connection failed".to_owned()
    } else {
        "the HTTP request failed".to_owned()
    }
}

#[cfg(test)]
#[path = "_tests_/error_diagnostic_tests.rs"]
mod error_diagnostic_tests;
