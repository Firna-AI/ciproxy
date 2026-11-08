//! Composition of the Linux guest from explicit runtime configuration.

use std::path::PathBuf;

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use clap::{Parser, Subcommand};

use crate::boat::BoatChannel;
use crate::boundary::{Channel, Power};
use crate::error::Result;
use crate::guest_setup::LinuxGuestSetup;
use crate::power::{LinuxPower, LinuxShutdownCommand, NamespacePower};
use crate::process::LinuxRunner;
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
    /// Configuration transport and shutdown mode.
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Receive newline-delimited protocol JSON from the host over vsock.
    Vsock {
        /// Guest port used by the own-machine agent's Firecracker connection.
        #[arg(long)]
        port: u32,
    },
    /// Consume one private launch file and POST the runner exit over HTTPS.
    Boat {
        /// Root-owned mode 0600 file in a private runtime directory.
        #[arg(long)]
        registration_file: PathBuf,
    },
}

/// The Linux application boundary, used only by the binary.
#[async_trait]
pub trait Application: Send + Sync {
    /// Construct the transport and supervise this fresh guest.
    async fn run(&self, cli: Cli) -> Result<()>;
}

/// Linux implementation of the guest application.
pub struct LinuxApplication;

#[async_trait]
impl Application for LinuxApplication {
    async fn run(&self, cli: Cli) -> Result<()> {
        let own_machine = matches!(cli.mode, Mode::Vsock { .. });
        let (channel, power): (Box<dyn Channel>, Box<dyn Power>) = match cli.mode {
            Mode::Vsock { port } => (
                Box::new(VsockChannel::accept(port).await?),
                Box::new(LinuxPower {
                    command: Box::new(LinuxShutdownCommand),
                }),
            ),
            Mode::Boat { registration_file } => (
                Box::new(BoatChannel::from_file(&registration_file)?),
                Box::new(NamespacePower),
            ),
        };
        let mut guest: Box<dyn Guest> = Box::new(Supervisor {
            image_id: ImageId(cli.image_id),
            channel,
            runner: Box::new(LinuxRunner::new(
                cli.runner_entry,
                own_machine,
                Box::new(LinuxGuestSetup),
            )),
            power,
        });
        guest.run().await
    }
}
