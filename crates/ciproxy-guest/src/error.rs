//! Typed guest failures with a redacted internal-error display.

use internal_error::InternalError;

/// Failure at the guest lifecycle boundary.
#[derive(Debug, thiserror::Error, internal_error::ErrorContract)]
pub enum Error {
    /// The runner registration file can be read by another user.
    #[error("[ciproxy-guest] registration file must be private")]
    InsecureRegistrationFile,
    /// A required Linux command ended unsuccessfully.
    #[error("[ciproxy-guest] poweroff command failed")]
    PoweroffFailed,
    /// The cause stays available to trusted callers through `Debug`.
    #[error("[ciproxy-guest] internal error")]
    Internal(#[from] InternalError),
}

/// Result of a guest operation.
pub type Result<T> = std::result::Result<T, Error>;
