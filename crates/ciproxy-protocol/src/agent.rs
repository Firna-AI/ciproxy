//! Tagged JSON messages on the agent's outbound HTTPS WebSocket.

use serde::{Deserialize, Serialize};

use crate::identifiers::{CommandId, JobId, MachineId, OrganizationId, RunnerId, WorkspaceId};
use crate::images::ImageDownload;
use crate::jobs::JobFacts;
use crate::reports::MachineReport;
use crate::runners::{RunnerExit, RunnerSpec};
use crate::secret::Secret;
use crate::version::ProtocolVersion;

/// Messages sent by an agent to the control plane.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentMessage {
    /// Register a new machine with its workspace's install token.
    Register {
        /// The wire version understood by this agent.
        protocol_version: ProtocolVersion,
        /// Install token whose authority identifies the workspace, hidden from diagnostics.
        install_token: Secret,
        /// Initial hardware, capacity, and agent state.
        report: MachineReport,
    },
    /// Authenticate a connection, including the state kept during an outage.
    Hello {
        /// The wire version understood by this agent.
        protocol_version: ProtocolVersion,
        /// Workspace that owns this registered machine.
        workspace_id: WorkspaceId,
        /// The machine registered by the gateway.
        machine_id: MachineId,
        /// Persistent credential of this machine within the named workspace.
        credential: Secret,
        /// Full current state for reconnect reconciliation.
        report: MachineReport,
    },
    /// Publish current state at the documented ten-second interval.
    Report {
        /// Full current state of this machine.
        report: MachineReport,
    },
    /// Report a runner ready to take its one job.
    RunnerReady {
        /// The runner that finished starting.
        runner_id: RunnerId,
    },
    /// Report the actual job that GitHub assigned to a runner.
    JobStarted {
        /// The runner that took the job.
        runner_id: RunnerId,
        /// The assigned GitHub job.
        job_id: JobId,
    },
    /// Report a runner process exit, also when it never took a job.
    RunnerExited {
        /// The runner that exited.
        runner_id: RunnerId,
        /// Process outcome, distinct from the GitHub job result.
        exit: RunnerExit,
    },
    /// Correlate a command with its outcome.
    CommandResult {
        /// Command to which this result belongs.
        command_id: CommandId,
        /// Whether the requested operation completed.
        outcome: CommandOutcome,
    },
}

/// Commands and connection replies sent by the control plane.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    /// Return the identity and credential of a newly registered machine.
    Registered {
        /// Workspace that owns the registered machine and its credential.
        workspace_id: WorkspaceId,
        /// Identity retained across later agent connections.
        machine_id: MachineId,
        /// Workspace-scoped machine credential retained privately by the machine.
        credential: Secret,
    },
    /// Confirm the version used by an authenticated connection.
    Welcome {
        /// Agreed wire version for the connection.
        protocol_version: ProtocolVersion,
    },
    /// Start a runner in a fresh VM from the newest local image.
    StartRunner {
        /// Identifier used for the result of this command.
        command_id: CommandId,
        /// Label, size, identity, and one-use registration.
        runner: RunnerSpec,
    },
    /// Stop an idle runner after it has been removed from GitHub.
    StopRunner {
        /// Identifier used for the result of this command.
        command_id: CommandId,
        /// Organization whose GitHub runner registration was removed.
        organization_id: OrganizationId,
        /// The idle runner to stop, never a running job.
        runner_id: RunnerId,
    },
    /// Supply scope inputs for the actual job assigned to a runner.
    JobFacts {
        /// Identifier used for the result of this command.
        command_id: CommandId,
        /// Runner that took the job.
        runner_id: RunnerId,
        /// Job to which the facts belong.
        job_id: JobId,
        /// Values whose presence alone grants no trust or write access.
        facts: JobFacts,
    },
    /// Finish running jobs, accept no new jobs, and stop warm runners.
    Drain {
        /// Identifier used for the result of this command.
        command_id: CommandId,
    },
    /// End the reversible drain and allow new and warm runners again.
    Resume {
        /// Identifier used for the result of this command.
        command_id: CommandId,
    },
    /// Finish running jobs and stop the agent after credential revocation.
    Remove {
        /// Identifier used for the result of this command.
        command_id: CommandId,
    },
    /// Clear this machine's cache entries without deleting Docker disks.
    ClearCache {
        /// Identifier used for the result of this command.
        command_id: CommandId,
    },
    /// Download and verify an image without changing running VMs.
    FetchImage {
        /// Identifier used for the result of this command.
        command_id: CommandId,
        /// Download and checksum metadata, with a redacted URL.
        image: ImageDownload,
    },
}

/// A command outcome with no free-form error that could contain a secret.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CommandOutcome {
    /// The requested operation completed.
    Applied,
    /// The operation failed, without changing the machine's other work.
    Failed {
        /// The operation that failed.
        reason: CommandFailure,
    },
}

/// Typed command failure facts, without credentials or job output.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandFailure {
    /// A runner could not start.
    RunnerStart,
    /// An idle runner could not stop.
    RunnerStop,
    /// Cache entries could not be cleared.
    CacheClear,
    /// An image download failed.
    ImageDownload,
    /// A downloaded image did not match its checksum.
    ImageVerification,
    /// An agent could not reach a guest over vsock.
    GuestConnection,
}
