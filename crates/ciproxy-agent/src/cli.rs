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
        /// Registration endpoint. A repair keeps the endpoint of the saved identity.
        #[arg(long)]
        agent_url: Option<AgentEndpoint>,
    },
    /// Run the machine agent under systemd.
    Run,
    /// Finish running jobs, discard their VMs, and remove the agent.
    Uninstall,
    /// Run one job VM. The agent starts this command in its own launchd job.
    #[cfg(target_os = "macos")]
    #[command(hide = true)]
    VmRun {
        /// The local ID of the VM.
        local_id: String,
    },
}

pub(crate) enum Action {
    Install {
        token: Secret,
        agent_url: Option<AgentEndpoint>,
    },
    Run,
    Uninstall,
    #[cfg(target_os = "macos")]
    VmRun {
        local_id: String,
    },
}

impl Cli {
    pub(crate) fn action(self) -> Action {
        match self.command {
            Some(Command::Install { token, agent_url }) => Action::Install {
                token: Secret::new(token),
                agent_url,
            },
            Some(Command::Uninstall) => Action::Uninstall,
            #[cfg(target_os = "macos")]
            Some(Command::VmRun { local_id }) => Action::VmRun { local_id },
            _ => Action::Run,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
