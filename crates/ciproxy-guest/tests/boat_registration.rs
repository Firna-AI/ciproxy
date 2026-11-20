//! The native Boat guest stops safely when no private input was delivered.

use std::process::Command;

#[test]
fn missing_boat_registration_reports_a_safe_error() {
    let directory = tempfile::tempdir().unwrap();
    let registration = directory.path().join("registration.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ciproxy-guest"))
        .args(["--image-id", "fixture-image", "boat", "--registration-file"])
        .arg(registration)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let diagnostics =
        String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
    assert!(diagnostics.contains("[ciproxy-guest] registration file is missing"));
    assert!(!diagnostics.contains(&directory.path().display().to_string()));
}
