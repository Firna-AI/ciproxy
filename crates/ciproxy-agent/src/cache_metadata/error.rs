//! Typed relay availability and private internal failures.

use internal_error::InternalError;

/// Root relay operations fail into the existing cache fallback.
#[derive(Debug, thiserror::Error, internal_error::ErrorContract)]
pub enum Error {
    /// The local or hosted metadata service is unavailable.
    #[error("[agent/cache-metadata] metadata service unavailable")]
    Unavailable,
    /// An unhandled source-preserving failure.
    #[error("[agent/cache-metadata] internal failure")]
    Internal(#[from] InternalError),
}

/// Root metadata relay result.
pub type Result<T> = std::result::Result<T, Error>;
