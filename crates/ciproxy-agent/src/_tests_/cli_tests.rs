//! Public command-line version output.

use clap::{CommandFactory, Parser, error::ErrorKind};

use super::{Action, Cli};

#[test]
fn explicit_commands_select_machine_lifecycle_actions() {
    let action = Cli::try_parse_from(["ciproxy-agent", "install", "--token", "install-secret"])
        .unwrap()
        .action();
    assert!(matches!(action, Action::Install(token) if token.expose_secret() == "install-secret"));
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
