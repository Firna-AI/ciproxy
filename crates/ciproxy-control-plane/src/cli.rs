//! Hosted serving by default, with an explicit image-sizing diagnostic.

use std::process::ExitCode;
use std::time::Duration;
use std::{env, io};

use ciproxy_gateway::logging;
use ciproxy_protocol::version::ProtocolVersion;
use clap::{Args, Parser, Subcommand};
use tokio::runtime::Builder;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

use ciproxy_control_plane::application::{ControlPlane, Production};
use ciproxy_control_plane::config::{self, Config};
use ciproxy_control_plane::error::Result;

#[derive(Parser)]
#[command(name = "ciproxy-control-plane", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the dashboard API, agent gateway, and hosted workers.
    Serve,
    /// Report diagnostic image sizing without starting the service.
    Diagnose(Diagnostic),
}

#[derive(Args)]
struct Diagnostic {
    /// Override the logical image bytes for this diagnostic only.
    #[arg(long, env = "CIPROXY_IMAGE_ROOT_BYTES")]
    image_root_bytes: Option<u64>,
}

impl Cli {
    async fn execute(self, application: &dyn ControlPlane) -> Result<()> {
        match self.command.unwrap_or(Command::Serve) {
            Command::Serve => application.serve(Config::from_env()?).await,
            Command::Diagnose(args) => {
                let root_sizing = config::root_sizing(args.image_root_bytes, None);
                tracing::info!(protocol_version = ?ProtocolVersion::CURRENT, image_root_bytes = root_sizing.image_root_bytes, "ciproxy control plane diagnostic");
                Ok(())
            }
        }
    }
}

pub(crate) fn main() -> ExitCode {
    let args = Cli::parse();
    let configured = logging::subscriber(env::var("RUST_LOG"), BoxMakeWriter::new(io::stdout));
    if let Some(notice) = configured.notice {
        eprintln!("{notice}");
    }
    configured.subscriber.init();
    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "control plane runtime could not start");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(args.execute(&Production));
    runtime.shutdown_timeout(Duration::from_secs(5));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "control plane stopped");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
