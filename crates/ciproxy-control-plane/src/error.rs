//! Typed hosted startup and orchestration failures.

use internal_error::InternalError;
use thiserror::Error;

/// Hosted failures never contain a credential or serialized frame.
#[derive(Debug, Error, internal_error::ErrorContract)]
pub enum Error {
    /// Required operator configuration is absent.
    #[error("[control-plane] missing setting {0}")]
    MissingSetting(&'static str),
    /// Operator configuration has the wrong form. The value stays out.
    #[error("[control-plane] malformed setting {0}")]
    MalformedSetting(&'static str),
    /// An image URL has no host, so no signature can name it.
    #[error("[control-plane] image URL has no host")]
    UnsignableUrl,
    /// A release file read returned an HTTP error status.
    #[error("[control-plane] image release read returned HTTP status {0}")]
    ReleaseStatus(u16),
    /// The release's `downloads.json` has no artifact under this key.
    #[error("[control-plane] downloads.json has no {0} artifact")]
    MissingArtifact(&'static str),
    /// A runner event does not belong to this authenticated machine.
    #[error("[control-plane] runner does not belong to this machine")]
    RunnerOwnership,
    /// A concurrent or previous owner changed the runtime's state.
    #[error("[control-plane] execution state changed; retry reconciliation")]
    StateChanged,
    /// An owned runtime task panicked or could not be joined.
    #[error("[control-plane] runtime task join failed")]
    TaskJoin(#[source] tokio::task::JoinError),
    /// Internal failure with its typed source retained.
    #[error("[control-plane] internal failure")]
    Internal(#[from] InternalError),
}

/// Hosted operation result.
pub type Result<T> = std::result::Result<T, Error>;
