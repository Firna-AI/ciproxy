//! A sandbox guest stops safely when no private input was delivered.

use std::process::Command;

#[test]
fn missing_sandbox_registration_reports_a_safe_error() {
    let directory = tempfile::tempdir().unwrap();
    let registration = directory.path().join("registration.json");
    for mode in ["sandbox", "boat"] {
        let output = Command::new(env!("CARGO_BIN_EXE_ciproxy-guest"))
            .args(["--image-id", "fixture-image", mode, "--registration-file"])
            .arg(&registration)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1), "{mode}");
        let diagnostics =
            String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
        assert!(
            diagnostics.contains("[ciproxy-guest] registration file is missing"),
            "{mode}"
        );
        assert!(
            !diagnostics.contains(&directory.path().display().to_string()),
            "{mode}"
        );
    }
}
