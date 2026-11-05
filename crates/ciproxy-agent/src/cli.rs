//! CLI used directly and by the shared release installer.

use ciproxy_protocol::secret::Secret;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ciproxy-agent", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Register this machine and install its systemd service.
    Install {
        /// Workspace install token. It works for one hour.
        #[arg(long)]
        token: String,
    },
    /// Run the machine agent under systemd.
    Run,
    /// Finish running jobs, discard their VMs, and remove the agent.
    Uninstall,
}

pub(crate) enum Action {
    Install(Secret),
    Run,
    Uninstall,
}

impl Cli {
    pub(crate) fn action(self) -> Action {
        match self.command {
            Some(Command::Install { token }) => Action::Install(Secret::new(token)),
            Some(Command::Uninstall) => Action::Uninstall,
            _ => Action::Run,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
