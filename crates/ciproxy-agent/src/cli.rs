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
    /// Prepare the Mac as install does, with no registration, and print the
    /// hardware report. For host checks. Uninstall removes what it makes.
    #[cfg(target_os = "macos")]
    #[command(hide = true)]
    PrepareHost {
        /// Also install and load the launchd daemon.
        #[arg(long)]
        with_service: bool,
    },
    /// Publish a local `macos.tar` as the newest image. A test and operator
    /// path: run it as root after prepare-host and before install.
    #[cfg(target_os = "macos")]
    #[command(hide = true)]
    ImportImage {
        /// The `macos.tar` file. On the area volume, the copy is an APFS clone.
        #[arg(long)]
        archive: std::path::PathBuf,
        /// The image ID that the agent reports.
        #[arg(long, value_parser = crate::macos::image_import::parse_image_id)]
        image_id: String,
        /// The SHA-256 of the archive, as 64 hex digits.
        #[arg(long, value_parser = crate::macos::image_import::parse_sha256)]
        sha256: String,
        /// The root disk length of the image in bytes.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        root_disk_bytes: u64,
    },
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
    #[cfg(target_os = "macos")]
    PrepareHost {
        with_service: bool,
    },
    #[cfg(target_os = "macos")]
    ImportImage(crate::macos::image_import::ImageImport),
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
            #[cfg(target_os = "macos")]
            Some(Command::PrepareHost { with_service }) => Action::PrepareHost { with_service },
            #[cfg(target_os = "macos")]
            Some(Command::ImportImage {
                archive,
                image_id,
                sha256,
                root_disk_bytes,
            }) => Action::ImportImage(crate::macos::image_import::ImageImport {
                archive,
                image_id: ciproxy_protocol::identifiers::ImageId(image_id),
                sha256,
                root_disk_bytes,
            }),
            _ => Action::Run,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
