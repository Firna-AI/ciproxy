//! Composition of the Linux or macOS guest from explicit runtime configuration.

use std::path::PathBuf;

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use clap::{Parser, Subcommand};

use crate::boundary::{Channel, Power};
use crate::error::Result;
use crate::guest_setup::{GuestSetup, LinuxGuestSetup, MacGuestSetup};
use crate::platform::{Account, Platform};
use crate::power::{SandboxPower, SystemShutdownCommand, VmPower};
use crate::process::StockRunner;
use crate::sandbox::SandboxChannel;
use crate::service::{Guest, Supervisor};
use crate::vsock::VsockChannel;

/// Options for one fresh guest. Registrations never use command-line flags.
#[derive(Parser)]
#[command(about = "Run one ephemeral GitHub Actions runner")]
pub struct Cli {
    /// Identifier recorded in the image's release manifest.
    #[arg(long)]
    image_id: String,
    /// Script that loads the image environment and executes the stock runner.
    #[arg(long, default_value = "/usr/local/libexec/ciproxy-runner")]
    runner_entry: PathBuf,
    /// Local account that runs the stock runner.
    #[arg(long, default_value = "runner")]
    runner_account: String,
    /// Configuration transport and shutdown mode.
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Receive newline-delimited protocol JSON from the host over vsock.
    Vsock {
        /// Guest port of the own-machine agent's vsock connection.
        #[arg(long)]
        port: u32,
    },
    /// Consume one private launch file and POST the runner exit over HTTPS.
    #[command(alias = "boat")]
    Sandbox {
        /// Root-owned mode 0600 file in a private runtime directory.
        #[arg(long)]
        registration_file: PathBuf,
    },
}

/// The guest application boundary, used only by the binary.
#[async_trait]
pub trait Application: Send + Sync {
    /// Construct the transport and supervise this fresh guest.
    async fn run(&self, cli: Cli) -> Result<()>;
}

/// Guest application for the platform of this build.
pub struct GuestApplication;

#[async_trait]
impl Application for GuestApplication {
    async fn run(&self, cli: Cli) -> Result<()> {
        let platform = Platform::current();
        let own_machine = matches!(cli.mode, Mode::Vsock { .. });
        let (channel, power): (Box<dyn Channel>, Box<dyn Power>) = match cli.mode {
            Mode::Vsock { port } => (
                Box::new(VsockChannel::accept(port).await?),
                Box::new(VmPower {
                    platform,
                    command: Box::new(SystemShutdownCommand),
                }),
            ),
            Mode::Sandbox { registration_file } => (
                Box::new(SandboxChannel::from_file(&registration_file)?),
                Box::new(SandboxPower),
            ),
        };
        let account = Account::lookup(&cli.runner_account)?;
        let setup: Box<dyn GuestSetup> = match platform {
            Platform::Linux => Box::new(LinuxGuestSetup),
            Platform::MacOs => Box::new(MacGuestSetup),
        };
        let mut guest: Box<dyn Guest> = Box::new(Supervisor {
            image_id: ImageId(cli.image_id),
            channel,
            runner: Box::new(StockRunner::new(
                platform,
                account,
                cli.runner_entry,
                own_machine,
                setup,
            )),
            power,
        });
        guest.run().await
    }
}
