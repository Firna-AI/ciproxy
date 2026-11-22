//! Entry point for the ciproxy job guest on Linux and macOS.

use std::process::ExitCode;

use ciproxy_guest::{Application, Cli, GuestApplication};
use clap::Parser;
use tracing::error;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let application: Box<dyn Application> = Box::new(GuestApplication);
    match application.run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(%error, cause = %error.diagnosis(), "guest stopped");
            ExitCode::FAILURE
        }
    }
}
