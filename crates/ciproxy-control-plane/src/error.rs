//! Typed hosted startup and orchestration failures.

use internal_error::InternalError;
use thiserror::Error;

/// Hosted failures never contain a credential or serialized frame.
#[derive(Debug, Error, internal_error::ErrorContract)]
pub enum Error {
    /// Required operator configuration is absent.
    #[error("[control-plane] missing setting {0}")]
    MissingSetting(&'static str),
    /// A runner event does not belong to this authenticated machine.
    #[error("[control-plane] runner does not belong to this machine")]
    RunnerOwnership,
    /// A concurrent or previous owner changed the runtime's state.
    #[error("[control-plane] execution state changed; retry reconciliation")]
    StateChanged,
    /// Internal failure with its typed source retained.
    #[error("[control-plane] internal failure")]
    Internal(#[from] InternalError),
}

/// Hosted operation result.
pub type Result<T> = std::result::Result<T, Error>;
