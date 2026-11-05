//! Transport failures cannot authorize a durable machine removal.

use ciproxy_protocol::agent::ControlMessage;
use internal_error::InternalError;
use thiserror::Error;

#[derive(Debug, Error, internal_error::ErrorContract)]
/// A connection failure never states whether the machine credential is valid.
pub enum ConnectionError {
    /// The stream is absent or closed without an authenticated rejection.
    #[error("[agent/connection] the connection closed")]
    Disconnected,
    /// Outbound control connections require TLS.
    #[error("[agent/connection] an HTTPS URL is required")]
    HttpsRequired,
    /// Transport diagnostics remain private.
    #[error("[agent/connection] internal error")]
    Internal(#[from] InternalError),
}

/// Result of transport work, separate from machine lifecycle failures.
pub(crate) type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug)]
/// A received control event, distinct from failed connection or send work.
pub enum ControlEvent {
    /// A control message received on this connection.
    Message(Box<ControlMessage>),
    /// The gateway rejected the machine credential after this socket sent Hello.
    CredentialRejected,
}
