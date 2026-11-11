//! Typed failures at the machine boundary. Internal diagnostics stay private.

use internal_error::InternalError;
use thiserror::Error;

#[derive(Debug, Error, internal_error::ErrorContract)]
/// Typed handled failures and a redacted internal fallback.
pub enum Error {
    /// The agent endpoint must be a secure WebSocket URL with a host.
    #[error("[agent] the agent endpoint must be a wss:// URL with a host")]
    AgentEndpoint,
    /// An owned VM network still has host resources after cleanup.
    #[error("[agent/network] network index {index} still has owned resources")]
    NetworkCleanup {
        /// Index whose cleanup must retry.
        index: u16,
    },
    /// Run this command as root on x64 Linux.
    #[error("[agent] run this command as root on x64 Linux")]
    HostUnsupported,
    /// Host preparation needs the named repair before it can continue safely.
    #[error("[agent/host] {component}: {repair}")]
    Preparation {
        /// Missing or unsafe host component.
        component: &'static str,
        /// Action that repairs this component.
        repair: &'static str,
    },
    /// Install the agent before starting it.
    #[error("[agent] install the agent before starting it")]
    NotInstalled,
    /// The machine credential was revoked.
    #[error("[agent] the machine credential was revoked")]
    Revoked,
    /// The machine identity does not match its workspace.
    #[error("[agent] the machine identity does not match its workspace")]
    IdentityMismatch,
    /// The connection closed.
    #[error("[agent] the connection closed")]
    Disconnected,
    /// The peer uses an unsupported protocol version.
    #[error("[agent] the peer uses an unsupported protocol version")]
    ProtocolVersion,
    /// The machine is draining or removed.
    #[error("[agent] the machine is draining or removed")]
    Draining,
    /// The runner is unknown, belongs to another organization, or has a job.
    #[error("[agent] the runner is unknown, belongs to another organization, or has a job")]
    RunnerMismatch,
    /// The machine has insufficient resources for this runner.
    #[error("[agent] the machine has insufficient resources for this runner")]
    Capacity,
    /// No verified image is available.
    #[error("[agent] no verified image is available")]
    NoImage,
    /// Only the active runtime can mutate the owned image cache.
    #[error("[agent] image cache ownership is unavailable")]
    ImageOwnership,
    /// The configured image reserve cannot hold this update or VM support file.
    #[error(
        "[agent/images] image reserve is full. Keep the current image or increase the configured image budget after draining."
    )]
    ImageBudget,
    /// The machine cache service is unavailable.
    #[error("[agent] the machine cache service is unavailable")]
    CacheUnavailable,
    /// The Docker filesystem failed its offline check.
    #[error("[agent/docker] the Docker filesystem failed its offline check")]
    DockerFilesystem,
    /// The image checksum or byte count does not match.
    #[error("[agent] the image checksum or byte count does not match")]
    ImageVerification,
    /// The image bundle must contain the released regular files only.
    #[error("[agent] the image bundle has an invalid released file layout")]
    ImageLayout,
    /// A host tool failed: {program}.
    #[error("[agent] a host tool failed: {program}")]
    HostCommand {
        /// Name of the host tool that failed.
        program: String,
    },
    /// An HTTPS URL is required.
    #[error("[agent] an HTTPS URL is required")]
    HttpsRequired,
    /// The guest sent an invalid message for this VM.
    #[error("[agent] the guest sent an invalid message for this VM")]
    GuestMismatch,
    /// Internal error.
    #[error("[agent] internal error")]
    Internal(#[from] InternalError),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
