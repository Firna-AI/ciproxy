//! Public command-line version output.

use clap::{CommandFactory, Parser, error::ErrorKind};

use super::{Action, Cli};

#[test]
fn explicit_commands_select_machine_lifecycle_actions() {
    let action = Cli::try_parse_from(["ciproxy-agent", "install", "--token", "install-secret"])
        .unwrap()
        .action();
    assert!(matches!(action, Action::Install { token, agent_url: None }
        if token.expose_secret() == "install-secret"));
    assert!(matches!(
        Cli::try_parse_from(["ciproxy-agent", "run"])
            .unwrap()
            .action(),
        Action::Run
    ));
    assert!(matches!(
        Cli::try_parse_from(["ciproxy-agent", "uninstall"])
            .unwrap()
            .action(),
        Action::Uninstall
    ));
}

#[test]
fn install_accepts_a_wss_endpoint_and_rejects_invalid_endpoints() {
    let action = Cli::try_parse_from([
        "ciproxy-agent",
        "install",
        "--token",
        "synthetic-token",
        "--agent-url",
        "wss://preview.ciproxy.example:8443/agent/",
    ])
    .unwrap()
    .action();
    assert!(
        matches!(action, Action::Install { token, agent_url: Some(endpoint) }
        if token.expose_secret() == "synthetic-token"
            && endpoint.as_str() == "wss://preview.ciproxy.example:8443/agent/")
    );
    for endpoint in [
        "ws://preview.example/agent/",
        "https://preview.example/agent/",
        "wss://",
        "wss://[invalid]/agent/",
    ] {
        let result = Cli::try_parse_from([
            "ciproxy-agent",
            "install",
            "--token",
            "synthetic-token",
            "--agent-url",
            endpoint,
        ])
        .map(|_| ())
        .map_err(|error| error.kind());
        assert_eq!(result, Err(ErrorKind::ValueValidation), "{endpoint}");
    }
}

#[test]
fn version_flag_reports_the_package_version() {
    let error = Cli::command()
        .try_get_matches_from(["ciproxy-agent", "--version"])
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::DisplayVersion);
    assert_eq!(
        error.to_string(),
        format!("ciproxy-agent {}\n", env!("CARGO_PKG_VERSION"))
    );
}
