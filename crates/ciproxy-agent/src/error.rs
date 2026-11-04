//! Typed failures at the machine boundary. Internal diagnostics stay private.

use internal_error::InternalError;
use thiserror::Error;

#[derive(Debug, Error, internal_error::ErrorContract)]
/// Typed handled failures and a redacted internal fallback.
pub enum Error {
    /// Run this command as root on x64 Linux.
    #[error("[agent] run this command as root on x64 Linux")]
    HostUnsupported,
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
    /// The machine cache service is unavailable.
    #[error("[agent] the machine cache service is unavailable")]
    CacheUnavailable,
    /// The image checksum or byte count does not match.
    #[error("[agent] the image checksum or byte count does not match")]
    ImageVerification,
    /// The image bundle must contain only vmlinux and rootfs.ext4.
    #[error("[agent] the image bundle must contain only vmlinux and rootfs.ext4")]
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
