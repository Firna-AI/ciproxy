//! What an HTTP answer means for the partial file.

use reqwest::header::CONTENT_RANGE;
use reqwest::{Response, StatusCode};

use crate::error::{Error, Result};

/// Where the body of an accepted answer starts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Answer {
    /// A `200` body: the whole archive from its first byte.
    Whole,
    /// A `206` body that starts at the end of the partial file.
    Rest,
    /// The answer to a `Range` request cannot continue the partial file:
    /// a `206` from another start, or a `416`. A request without `Range`
    /// can still get the whole archive.
    Unusable,
}

/// Accept a `200`, or a `206` whose `Content-Range` starts at `offset`.
/// Another `206` or a `416` to a `Range` request is unusable. Another HTTP
/// error status ends the command, and so does any other answer.
pub(super) fn read_answer(response: &Response, offset: u64) -> Result<Answer> {
    let status = response.status();
    if offset > 0 && status == StatusCode::RANGE_NOT_SATISFIABLE {
        return Ok(Answer::Unusable);
    }
    if status.is_client_error() || status.is_server_error() {
        return Err(Error::ImageStatus {
            status: status.as_u16(),
        });
    }
    if status == StatusCode::OK {
        return Ok(Answer::Whole);
    }
    let starts_at_offset = response
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes "))
        .and_then(|value| value.split_once('-'))
        .is_some_and(|(start, _)| start.parse::<u64>().ok() == Some(offset));
    match (
        status == StatusCode::PARTIAL_CONTENT && offset > 0,
        starts_at_offset,
    ) {
        (true, true) => Ok(Answer::Rest),
        (true, false) => Ok(Answer::Unusable),
        (false, _) => Err(Error::ImageAnswer),
    }
}
