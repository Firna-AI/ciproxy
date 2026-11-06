//! Tagged messages between the agent and its own job guests over vsock.

use serde::{Deserialize, Serialize};

use crate::cache::{CacheConnection, DockerDiskAttachment};
use crate::identifiers::{ImageId, JobId, OrganizationId, RunnerId};
use crate::jobs::JobFacts;
use crate::runners::RunnerExit;
use crate::runtime::RuntimeCredentials;
use crate::secret::Secret;
use crate::version::ProtocolVersion;

/// Explicit local preparation result, without credentials or guessed measurements.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobPreparation {
    /// The worker uses a bound cache proxy for this assignment.
    pub cache_ready: bool,
    /// The guest mounted the separately reserved Docker filesystem.
    pub docker_attached: bool,
}

/// Messages passed from an agent into its own fresh guest VM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToGuest {
    /// Resume action setup after the host prepared this job's local resources.
    PrepareJob {
        /// Job held at the trusted pre-action boundary.
        job_id: JobId,
        /// Trusted hosted assignment facts supplied to the machine cache.
        facts: JobFacts,
        /// This job's machine session, absent during GitHub fallback.
        cache: Option<CacheConnection>,
        /// Attached Docker drive, absent when unavailable or not included.
        docker: Option<DockerDiskAttachment>,
    },
    /// Configure the single runner before it connects to GitHub.
    ConfigureRunner {
        /// Wire version used over this vsock connection.
        protocol_version: ProtocolVersion,
        /// Identity of the runner owned by this guest.
        runner_id: RunnerId,
        /// Organization whose scale set issued the runner registration.
        organization_id: OrganizationId,
        /// One-use registration, passed over vsock rather than a disk.
        registration: Secret,
        /// Access to this machine's cache, when available.
        cache: Option<CacheConnection>,
    },
    /// Pass scope inputs after GitHub assigns the actual job.
    JobFacts {
        /// Job whose cache requests use these facts.
        job_id: JobId,
        /// Scope values, without choosing their source or trust.
        facts: JobFacts,
    },
    /// Make a repository Docker disk available after a job starts.
    DockerDisk {
        /// Job for which the agent attached this disk.
        job_id: JobId,
        /// Attached drive and the mode already selected by the agent.
        disk: DockerDiskAttachment,
    },
    /// Shut down a guest whose idle runner or completed job has stopped.
    Shutdown,
}

/// Guest observations sent to its agent over vsock.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuestToAgent {
    /// Pause before job code and supply the original trusted runtime assignment.
    JobStarting {
        /// Runner owned by this VM.
        runner_id: RunnerId,
        /// The scale-message job UUID from the runner's assignment.
        job_id: JobId,
        /// Original in-memory runtime handoff, never sent to the control plane.
        runtime: RuntimeCredentials,
    },
    /// Confirm local proxy readiness and Docker preparation before action setup.
    JobPrepared {
        /// Runner owned by this VM.
        runner_id: RunnerId,
        /// Prepared assignment.
        job_id: JobId,
        /// Actual preparation status before action setup continues.
        outcome: JobPreparation,
    },
    /// Announce a booted image before runner configuration.
    Hello {
        /// Wire version supported by this guest.
        protocol_version: ProtocolVersion,
        /// Image from which this VM booted.
        image_id: ImageId,
    },
    /// Report that the runner is ready to take a job.
    RunnerReady {
        /// Runner whose startup completed.
        runner_id: RunnerId,
    },
    /// Identify the job taken by the runner, without asserting its scope.
    JobStarted {
        /// Runner that took its one job.
        runner_id: RunnerId,
        /// GitHub identity to correlate with control-plane job facts.
        job_id: JobId,
    },
    /// Report the runner exit so the agent can tear down its fresh VM.
    RunnerExited {
        /// Runner whose process ended.
        runner_id: RunnerId,
        /// Process outcome, also available when no job started.
        exit: RunnerExit,
    },
}
