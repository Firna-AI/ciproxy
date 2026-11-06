//! The public command keeps diagnostic overrides outside the serving process.

use std::process::Command;

fn plain_output(bytes: Vec<u8>) -> String {
    let mut escape = false;
    String::from_utf8(bytes)
        .unwrap()
        .chars()
        .filter(|character| {
            if *character == '\u{1b}' {
                escape = true;
                return false;
            }
            if escape {
                escape = *character != 'm';
                return false;
            }
            true
        })
        .collect()
}

#[test]
fn diagnostic_command_keeps_explicit_and_environment_overrides_without_runtime_secrets() {
    for (arguments, expected) in [
        (vec!["diagnose"], "image_root_bytes=59000000000"),
        (
            vec!["diagnose", "--image-root-bytes", "63000000000"],
            "image_root_bytes=63000000000",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ciproxy-control-plane"))
            .args(arguments)
            .env_clear()
            .env("RUST_LOG", "info")
            .env("CIPROXY_IMAGE_ROOT_BYTES", "59000000000")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = plain_output(output.stdout);
        assert!(
            stdout.contains("ciproxy control plane diagnostic"),
            "{stdout}"
        );
        assert!(stdout.contains(expected), "{stdout}");
    }
}

#[test]
fn default_and_explicit_serving_ignore_diagnostic_overrides_and_require_runtime_configuration() {
    for arguments in [vec![], vec!["serve"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_ciproxy-control-plane"))
            .args(arguments)
            .env_clear()
            .env("RUST_LOG", "info")
            .env("CIPROXY_IMAGE_ROOT_BYTES", "invalid-diagnostic-value")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("missing setting DATABASE_URL"), "{stdout}");
        assert!(!stdout.contains("control plane diagnostic"), "{stdout}");
    }
}
