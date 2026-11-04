//! Typed failures and source-preserving IO conversion for repository checks.

use std::io;
use std::panic::Location;
use std::process::ExitStatus;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("[xtask/check] command {program} failed: {status}")]
    CommandFailed { program: String, status: ExitStatus },
    #[error("[xtask/check] {count} phase(s) failed")]
    PhasesFailed { count: usize },
    #[error("[xtask/file_length] {count} files exceed 300 lines")]
    FileLength { count: usize },
    #[error("[xtask/markdown] {count} broken local links")]
    MarkdownLinks { count: usize },
    #[error("[xtask/internal] {0}")]
    Internal(#[from] InternalError),
}

#[derive(Debug, thiserror::Error)]
#[error("IO failure at {location}: {source}")]
pub(crate) struct InternalError {
    #[source]
    source: io::Error,
    location: &'static Location<'static>,
}

/// Captures the caller while retaining the original IO error.
pub(crate) trait IoResultExt<T> {
    #[track_caller]
    fn internal(self) -> Result<T>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    #[track_caller]
    fn internal(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(Error::Internal(InternalError {
                source,
                location: Location::caller(),
            })),
        }
    }
}
