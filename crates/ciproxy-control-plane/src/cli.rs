//! Minimal command line for the control plane skeleton.

use std::{env, io};

use ciproxy_gateway::logging;
use ciproxy_protocol::version::ProtocolVersion;
use clap::Parser;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(name = "ciproxy-control-plane", version)]
struct Cli {}

pub(crate) fn main() {
    let _args = Cli::parse();
    let configured = logging::subscriber(env::var("RUST_LOG"), BoxMakeWriter::new(io::stdout));
    if let Some(notice) = configured.notice {
        eprintln!("{notice}");
    }
    configured.subscriber.init();
    tracing::info!(protocol_version = ?ProtocolVersion::CURRENT, "ciproxy control plane skeleton");
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
