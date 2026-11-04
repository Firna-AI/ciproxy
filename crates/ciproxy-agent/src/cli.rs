//! Minimal command line for the agent skeleton.

use ciproxy_protocol::version::ProtocolVersion;
use clap::Parser;

#[derive(Parser)]
#[command(name = "ciproxy-agent", version)]
struct Cli {}

pub(crate) fn main() {
    let _args = Cli::parse();
    tracing_subscriber::fmt::init();
    tracing::info!(protocol_version = ?ProtocolVersion::CURRENT, "ciproxy agent skeleton");
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
