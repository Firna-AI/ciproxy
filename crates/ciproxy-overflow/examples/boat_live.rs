//! Opt-in live Boat account, snapshot, guest, recovery, usage, and cleanup probe.

mod live_boat;
mod live_support;

use std::path::PathBuf;

use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::provider::BackendFactory;
use clap::{Parser, Subcommand};

use live_support::SafeResult;

#[derive(Parser)]
struct Arguments {
    #[arg(long)]
    boat_key_file: PathBuf,
    #[arg(long, default_value = "personal")]
    wallet: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Account,
    Create {
        #[arg(long)]
        state_file: PathBuf,
        #[arg(long)]
        snapshot: String,
        #[arg(long, default_value = "large")]
        size: String,
        #[arg(long)]
        trial: bool,
    },
    Recover {
        #[arg(long)]
        state_file: PathBuf,
    },
    Runner {
        #[arg(long)]
        state_file: PathBuf,
        #[arg(long)]
        registration_file: PathBuf,
    },
    Inspect {
        #[arg(long)]
        state_file: PathBuf,
    },
    Usage {
        #[arg(long)]
        state_file: PathBuf,
    },
    Delete {
        #[arg(long)]
        state_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> live_support::Result<()> {
    let args = Arguments::parse();
    let key = live_support::secret(&args.boat_key_file)?;
    let backend = BoatFactory::new()
        .backend(&key, &args.wallet)
        .safe("configure Boat")?;
    live_boat::run(backend, args.command).await
}
