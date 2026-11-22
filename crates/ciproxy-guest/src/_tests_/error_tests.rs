//! The guest log names the context and OS cause, but never quoted input.

use crate::error::{ErrorResultExt, Result};

fn bind() -> Result<()> {
    Err(std::io::Error::from_raw_os_error(libc::ENODEV)).context("bind guest vsock")
}

#[test]
fn diagnosis_names_the_context_and_the_os_error() {
    let error = bind().unwrap_err();
    let diagnosis = error.diagnosis();
    assert!(diagnosis.starts_with("bind guest vsock: "), "{diagnosis}");
    assert!(diagnosis.contains("os error"), "{diagnosis}");
    assert_eq!(error.to_string(), "[ciproxy-guest] internal error");
}

#[test]
fn diagnosis_withholds_causes_that_can_quote_input() {
    let decoded: std::result::Result<u32, _> = serde_json::from_str("\"jit-secret\"");
    let error = decoded.context("decode registration").unwrap_err();
    let diagnosis = error.diagnosis();
    assert_eq!(diagnosis, "decode registration: cause withheld");
    assert!(!diagnosis.contains("jit-secret"));
}
