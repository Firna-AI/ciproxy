//! Source-preserving errors shared by store implementations.

use std::error;
use std::fmt;
use std::panic::Location;

/// A failure from a store operation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested record does not exist in the organization.
    #[error("[store] record not found")]
    NotFound,
    /// A unique record already exists.
    #[error("[store] record already exists")]
    Conflict,
    /// A referenced record does not belong to the organization.
    #[error("[store] invalid record reference")]
    InvalidReference,
    /// A value conflicts with a stored requirement.
    #[error("[store] invalid stored value")]
    InvalidValue,
    /// Required Boat cost facts cannot be represented as a complete price interval.
    #[error("[store] Boat accounting is incomplete")]
    AccountingIncomplete,
    /// An implementation failed for an unhandled reason.
    #[error("[store/internal] {0}")]
    Internal(#[from] InternalError),
}

/// The result of a store operation.
pub type Result<T> = std::result::Result<T, Error>;

/// An original failure and its call site, with safe diagnostic output.
#[derive(thiserror::Error)]
#[error("storage operation failed at {location}")]
pub struct InternalError {
    #[source]
    source: Box<dyn error::Error + Send + Sync>,
    location: &'static Location<'static>,
}

impl fmt::Debug for InternalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InternalError")
            .field("location", &self.location)
            .finish_non_exhaustive()
    }
}

impl Error {
    /// Retain the source without exposing database values in diagnostics.
    #[track_caller]
    pub fn internal(source: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self::Internal(InternalError {
            source: source.into(),
            location: Location::caller(),
        })
    }
}

/// Capture an implementation failure at its caller.
pub trait ResultExt<T> {
    /// Preserve the original source and call site.
    #[track_caller]
    fn internal(self) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    #[track_caller]
    fn internal(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(Error::internal(source)),
        }
    }
}
