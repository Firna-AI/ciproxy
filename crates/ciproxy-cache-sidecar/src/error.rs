//! Source-preserving failures at cache IO boundaries.

use std::error::Error;
use std::panic::Location;

/// Result of a cache IO operation.
pub type Result<T> = std::result::Result<T, CacheError>;

/// Failures that the HTTP boundary converts to fallback or a cache miss.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// The session is absent or its credential is wrong.
    #[error("[cache/session] unauthorized session")]
    Unauthorized,
    /// An entry or transfer is no longer available.
    #[error("[cache/storage] cache entry unavailable")]
    Unavailable,
    /// The local tier cannot safely store this transfer.
    #[error("[cache/storage] local cache budget unavailable")]
    Budget,
    /// An unhandled IO or serialization failure.
    #[error("[cache/internal] {0}")]
    Internal(#[from] InternalError),
}

/// Original source and call site of an internal failure.
#[derive(Debug, thiserror::Error)]
#[error("failure at {location}: {source}")]
pub struct InternalError {
    #[source]
    source: Box<dyn Error + Send + Sync>,
    location: &'static Location<'static>,
}

pub(crate) trait ResultExt<T> {
    #[track_caller]
    fn internal(self) -> Result<T>;
}

impl<T, E: Error + Send + Sync + 'static> ResultExt<T> for std::result::Result<T, E> {
    #[track_caller]
    fn internal(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(CacheError::Internal(InternalError {
                source: Box::new(source),
                location: Location::caller(),
            })),
        }
    }
}
