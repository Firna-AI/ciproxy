//! Linux entry point for the ciproxy job guest.

use std::process::ExitCode;

use ciproxy_guest::{Application, Cli, LinuxApplication};
use clap::Parser;
use tracing::error;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let application: Box<dyn Application> = Box::new(LinuxApplication);
    match application.run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(%error, "guest stopped");
            ExitCode::FAILURE
        }
    }
}
