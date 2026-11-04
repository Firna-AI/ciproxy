//! Cache sidecar command-line composition root.

mod cli;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt().init();
    match cli::run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "Cache sidecar stopped");
            std::process::ExitCode::FAILURE
        }
    }
}
