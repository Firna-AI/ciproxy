//! Executable log selection without changing the test process environment.

use std::process::Command;

#[test]
fn rust_log_off_suppresses_the_hosted_startup_log() {
    let output = Command::new(env!("CARGO_BIN_EXE_ciproxy-control-plane"))
        .env("RUST_LOG", "off")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}
