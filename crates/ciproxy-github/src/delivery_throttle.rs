//! Provider rate-limit metadata for App webhook delivery operations.

use chrono::DateTime;
use reqwest::{Response, StatusCode};

use crate::error::Error;

/// Preserve rate-limit instructions from a failed delivery API response.
pub(crate) fn response_error(response: &Response) -> Error {
    let headers = response.headers();
    let retry_after = headers.get("retry-after");
    let exhausted = headers
        .get("x-ratelimit-remaining")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim() == "0");
    let status = response.status();
    if status == StatusCode::TOO_MANY_REQUESTS
        || status == StatusCode::FORBIDDEN && (retry_after.is_some() || exhausted)
    {
        let retry_after_seconds = retry_after
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse().ok());
        let reset_at = exhausted
            .then(|| headers.get("x-ratelimit-reset"))
            .flatten()
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse().ok())
            .and_then(|seconds| DateTime::from_timestamp(seconds, 0));
        Error::DeliveryThrottled {
            retry_after_seconds,
            reset_at,
        }
    } else {
        Error::AccessUnavailable(status.as_u16())
    }
}
