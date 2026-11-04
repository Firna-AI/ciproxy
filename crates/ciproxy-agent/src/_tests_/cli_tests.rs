//! Public command-line version output.

use clap::{CommandFactory, error::ErrorKind};

use super::Cli;

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
