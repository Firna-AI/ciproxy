//! CLI used directly and by the shared release installer.

use ciproxy_protocol::secret::Secret;
use clap::{Parser, Subcommand};

use crate::endpoint::AgentEndpoint;

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
        /// Control-plane endpoint. Install saves this wss:// URL for the service.
        #[arg(long)]
        agent_url: Option<AgentEndpoint>,
    },
    /// Run the machine agent under systemd.
    Run,
    /// Finish running jobs, discard their VMs, and remove the agent.
    Uninstall,
}

pub(crate) enum Action {
    Install {
        token: Secret,
        agent_url: Option<AgentEndpoint>,
    },
    Run,
    Uninstall,
}

impl Cli {
    pub(crate) fn action(self) -> Action {
        match self.command {
            Some(Command::Install { token, agent_url }) => Action::Install {
                token: Secret::new(token),
                agent_url,
            },
            Some(Command::Uninstall) => Action::Uninstall,
            _ => Action::Run,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
