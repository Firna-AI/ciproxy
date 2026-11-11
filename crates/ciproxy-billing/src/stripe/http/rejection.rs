//! Distinguish documented parameter validation from ambiguous mutation failures.

use ciproxy_store_interface::billing::payments::{ValidationFailure, ValidationRejection};
use reqwest::blocking::Response;
use serde::Deserialize;

#[derive(Deserialize)]
struct Envelope {
    error: Failure,
}

#[derive(Deserialize)]
struct Failure {
    #[serde(rename = "type")]
    kind: Kind,
    code: Option<Code>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    InvalidRequestError,
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Code {
    ParameterMissing,
    ParameterUnknown,
    ResourceMissing,
    #[serde(other)]
    Other,
}

pub(super) fn validation(response: Response, status: u16) -> Option<ValidationRejection> {
    let failure = response.json::<Envelope>().ok()?.error;
    if !matches!(failure.kind, Kind::InvalidRequestError) {
        return None;
    }
    let reason = match failure.code? {
        Code::ParameterMissing => ValidationFailure::MissingParameter,
        Code::ParameterUnknown => ValidationFailure::UnknownParameter,
        Code::ResourceMissing => ValidationFailure::MissingResource,
        Code::Other => return None,
    };
    Some(ValidationRejection { status, reason })
}
