//! Typed failures with diagnostic output that cannot expose wire secrets.

use std::fmt;

use internal_error::InternalError;

/// A gateway boundary failure.
#[derive(thiserror::Error, internal_error::ErrorContract)]
pub enum Error {
    /// The caller does not hold the workspace Admin role.
    #[error("[gateway] workspace Admin role required")]
    Forbidden,
    /// A token, machine, or workspace did not authenticate.
    #[error("[gateway] authentication failed")]
    Unauthorized,
    /// The agent uses a protocol version outside the supported window.
    #[error("[gateway] unsupported protocol version")]
    UnsupportedVersion,
    /// A connection was replaced by a newer authenticated connection.
    #[error("[gateway] connection replaced")]
    Replaced,
    /// A command targets a machine that has been removed or does not exist.
    #[error("[gateway] machine not found")]
    NotFound,
    /// A message is not valid at this stage of the connection.
    #[error("[gateway] unexpected message")]
    UnexpectedMessage,
    /// A malformed identifier or JSON message cannot be processed.
    #[error("[gateway] invalid message")]
    InvalidMessage,
    /// An unhandled failure retains its original source for trusted callers.
    #[error("[gateway] internal operation failed")]
    Internal(#[from] InternalError),
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// Result at a gateway boundary.
pub type Result<T> = std::result::Result<T, Error>;
