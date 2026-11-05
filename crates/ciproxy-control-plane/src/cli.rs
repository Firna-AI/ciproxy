//! Minimal command line for the control plane skeleton.

use std::{env, io};

use ciproxy_gateway::logging;
use ciproxy_protocol::version::ProtocolVersion;
use clap::Parser;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config;

#[derive(Parser)]
#[command(name = "ciproxy-control-plane", version)]
struct Cli {
    /// Logical bytes of the deployed image root, until its release manifest supplies them.
    #[arg(long, env = "CIPROXY_IMAGE_ROOT_BYTES")]
    image_root_bytes: Option<u64>,
}

pub(crate) fn main() {
    let args = Cli::parse();
    let root_sizing = config::root_sizing(args.image_root_bytes, None);
    let configured = logging::subscriber(env::var("RUST_LOG"), BoxMakeWriter::new(io::stdout));
    if let Some(notice) = configured.notice {
        eprintln!("{notice}");
    }
    configured.subscriber.init();
    tracing::info!(protocol_version = ?ProtocolVersion::CURRENT, image_root_bytes = root_sizing.image_root_bytes, "ciproxy control plane skeleton");
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
