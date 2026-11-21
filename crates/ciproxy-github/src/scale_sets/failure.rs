//! Resource-specific SDK failure classification without treating auth as absence.

use reqwest::{StatusCode, Url};

#[derive(Clone, Copy)]
pub(super) enum Operation<'a> {
    ScaleSet(i64),
    Open(i64),
    Jit(i64),
    Runner(i64),
    Session(i64, &'a str),
    Queue(&'a str),
}

pub(super) fn missing(error: &scaleset::Error, operation: Operation<'_>) -> bool {
    (error.is_not_found()
        || error.is_runner_not_found()
        || error.status() == Some(StatusCode::NO_CONTENT))
        && matches(error, operation)
}

pub(super) fn matches(error: &scaleset::Error, operation: Operation<'_>) -> bool {
    let decode_url = if let scaleset::Error::Decode { url, .. } = error {
        Some(url.as_str())
    } else {
        None
    };
    let source = error.api_error().map(|api| api.url.as_str()).or(decode_url);
    let Some(source) = source else {
        return false;
    };
    let Ok(url) = Url::parse(source) else {
        return false;
    };
    if let Operation::Queue(expected) = operation {
        let Ok(expected) = Url::parse(expected) else {
            return false;
        };
        return url.origin() == expected.origin() && url.path() == expected.path();
    }
    let expected = match operation {
        Operation::ScaleSet(id) => format!("/_apis/runtime/runnerscalesets/{id}"),
        Operation::Open(id) => format!("/_apis/runtime/runnerscalesets/{id}/sessions"),
        Operation::Jit(id) => format!("/_apis/runtime/runnerscalesets/{id}/generatejitconfig"),
        Operation::Runner(id) => format!("/_apis/distributedtask/pools/0/agents/{id}"),
        Operation::Session(id, session) => {
            format!("/_apis/runtime/runnerscalesets/{id}/sessions/{session}")
        }
        Operation::Queue(_) => return false,
    };
    url.path().ends_with(&expected)
}
