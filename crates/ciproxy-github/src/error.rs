//! Safe, source-preserving failures for GitHub integration boundaries.

use std::error;
use std::fmt;
use std::panic::Location;

/// A failure that callers can handle without inspecting text.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The webhook did not authenticate with the configured secret.
    #[error("[github] webhook signature is invalid")]
    InvalidSignature,
    /// GitHub cannot authorize this operation with the current installation.
    #[error("[github] installation access is unavailable: HTTP {0}")]
    AccessUnavailable(u16),
    /// GitHub has not made a new installation visible yet.
    #[error("[github] installation is not visible yet")]
    InstallationPending,
    /// Another listener owns the scale set's database session lock.
    #[error("[github] scale set session is already owned")]
    SessionOwned,
    /// The saved remote binding changed while this listener owned a session.
    #[error("[github] restart scale-set listener from {opened} to {current}")]
    ScaleSetRestart {
        /// The remote ID used by the live session.
        opened: i64,
        /// The remote ID now saved for the same internal scale set.
        current: i64,
    },
    /// GitHub no longer has the remote scale set; reconcile before retry.
    #[error("[github] scale set {scale_set_id} is missing")]
    ScaleSetMissing {
        /// The missing remote identifier.
        scale_set_id: i64,
    },
    /// A live remote resource would need a move that has not been proven.
    #[error(
        "[github] scale set {scale_set_id} belongs to group {actual_group}, expected {expected_group}"
    )]
    ScaleSetGroupChanged {
        /// The retained remote scale set.
        scale_set_id: i64,
        /// The managed group of the organization now.
        expected_group: i64,
        /// The different group still reported by GitHub.
        actual_group: i64,
    },
    /// A message omits metadata required to persist correct demand.
    #[error("[github] job message has no scale-set assignment time")]
    MissingAssignmentTime,
    /// A stale listener cannot start runners after removal or suspension.
    #[error("[github] organization cannot receive runners")]
    OrganizationInactive,
    /// A verified registration already belongs to another workspace.
    #[error("[github] organization belongs to another workspace")]
    WorkspaceConflict,
    /// GitHub refused to remove a runner that is executing a job.
    #[error("[github] runner is executing a job")]
    RunnerBusy,
    /// An unhandled source failure, with redacted diagnostic output.
    #[error("[github/internal] {0}")]
    Internal(#[from] InternalError),
}

/// A result from the GitHub integration.
pub type Result<T> = std::result::Result<T, Error>;

/// An original error whose display never includes credentials or bodies.
#[derive(thiserror::Error)]
#[error("GitHub operation failed at {location}")]
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

/// Capture the original source at a safe boundary.
pub trait ResultExt<T> {
    /// Preserve source and caller without exposing source text in diagnostics.
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
            Err(source) => Err(Error::Internal(InternalError {
                source: source.into(),
                location: Location::caller(),
            })),
        }
    }
}
