//! Minimal command line for the control plane skeleton.

use ciproxy_gateway::logging;
use ciproxy_protocol::version::ProtocolVersion;
use clap::Parser;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(name = "ciproxy-control-plane", version)]
struct Cli {}

pub(crate) fn main() {
    let _args = Cli::parse();
    logging::protect(Box::new(tracing_subscriber::fmt().finish())).init();
    tracing::info!(protocol_version = ?ProtocolVersion::CURRENT, "ciproxy control plane skeleton");
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
