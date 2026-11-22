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
}

/// Accept a `200`, or a `206` whose `Content-Range` starts at `offset`.
/// An HTTP error status ends the command, and so does any other answer.
pub(super) fn read_answer(response: &Response, offset: u64) -> Result<Answer> {
    let status = response.status();
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
    if status == StatusCode::PARTIAL_CONTENT && offset > 0 && starts_at_offset {
        Ok(Answer::Rest)
    } else {
        Err(Error::ImageAnswer)
    }
}
