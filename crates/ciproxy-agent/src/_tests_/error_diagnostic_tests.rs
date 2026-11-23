//! The log text of an error names its cause and holds no secret.

use std::io;

use super::diagnostic;
use crate::error::{Error, ErrorResultExt, Result};

fn failed<T>(result: Result<T>) -> Error {
    match result {
        Ok(_) => panic!("the call must fail"),
        Err(error) => error,
    }
}

#[test]
fn an_os_error_shows_its_contexts_and_its_message() {
    let denied: std::result::Result<(), io::Error> = Err(io::Error::from_raw_os_error(13));
    let error = failed(denied.context("list images").context("recover the images"));
    let text = diagnostic(&error);
    assert!(
        text.starts_with("[agent] internal error: recover the images: list images: "),
        "{text}"
    );
    assert!(text.contains("(os error 13)"), "{text}");
    assert!(text.contains("error_diagnostic_tests.rs"), "{text}");
}

#[test]
fn a_custom_io_message_shows_only_its_kind() {
    let custom: std::result::Result<(), io::Error> = Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "token=secret-value",
    ));
    let text = diagnostic(&failed(custom.context("read a file")));
    assert!(text.contains("read a file: PermissionDenied"), "{text}");
    assert!(!text.contains("secret-value"), "{text}");
}

#[test]
fn a_typed_error_keeps_its_text_through_a_context() {
    let typed: Result<()> = Err(Error::ImageLayout);
    assert_eq!(
        diagnostic(&failed(typed.context("publish the image"))),
        "[agent] the image bundle has an invalid released file layout"
    );
}

#[test]
fn a_json_error_shows_its_position_and_other_causes_stay_hidden() {
    let json: std::result::Result<u64, serde_json::Error> = serde_json::from_str("{");
    let text = diagnostic(&failed(json.context("decode")));
    assert!(text.contains("decode: JSON "), "{text}");

    #[derive(Debug, thiserror::Error)]
    #[error("https://images.example/macos.tar?signature=secret-value")]
    struct Other;
    let other: std::result::Result<(), Other> = Err(Other);
    let text = diagnostic(&failed(other.context("download")));
    assert!(text.contains("download: another cause"), "{text}");
    assert!(!text.contains("secret-value"), "{text}");
}

#[test]
fn a_typed_error_logs_its_display_text() {
    assert_eq!(
        diagnostic(&Error::HttpsRequired),
        "[agent] an HTTPS URL is required"
    );
}
