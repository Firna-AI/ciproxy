//! Typed caller outcomes and source-preserving redacted internal errors.

use std::{error, fmt, panic::Location};

/// A failure the overflow caller can handle without inspecting error text.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Another delivery owns the workspace session.
    #[error("[overflow/service] workspace is busy")]
    Busy,
    /// Boat did not accept the submitted credential.
    #[error("[overflow/account] Boat rejected the key")]
    RejectedKey,
    /// The required spend limit is not above zero.
    #[error("[overflow/account] spend limit must be above zero")]
    InvalidSpendLimit,
    /// This executor accepts Boat actions only.
    #[error("[overflow/service] action does not start a Boat runner")]
    NotBoatAction,
    /// A replay changed an immutable delivery identity.
    #[error("[overflow/service] delivery identity conflicts")]
    DeliveryConflict,
    /// A callback token is absent, consumed, or belongs to another runner.
    #[error("[overflow/callback] token is invalid or consumed")]
    InvalidToken,
    /// A cleanup target has no proof of ciproxy ownership.
    #[error("[overflow/cleanup] sandbox ownership is unknown")]
    OwnershipUnknown,
    /// GitHub still has a running job or still needs the idle registration.
    #[error("[overflow/cleanup] runner must be retained")]
    RunnerNeeded,
    /// A missing accepted fee or runtime interval prevents a safe spend decision.
    #[error("[overflow/accounting] current-month accounting is incomplete")]
    AccountingIncomplete,
    /// A provider read or idempotent operation can be retried.
    #[error("[overflow/provider] provider is unavailable")]
    ProviderUnavailable,
    /// A provider returned an incomplete or inconsistent response.
    #[error("[overflow/provider] provider response is incomplete")]
    InvalidProviderData,
    /// KMS rejected its authenticated request.
    #[error("[overflow/kms] request failed with HTTP {status}")]
    KmsHttp {
        /// Safe HTTP status, without the response body.
        status: u16,
    },
    /// KMS transport checks did not confirm the exact ciphertext or plaintext.
    #[error("[overflow/kms] integrity check failed")]
    KmsIntegrity,
    /// An unhandled failure whose diagnostics omit its source values.
    #[error("[overflow/internal] {0}")]
    Internal(#[from] InternalError),
}

/// A result from the overflow execution boundary.
pub type Result<T> = std::result::Result<T, Error>;

/// A source retained for diagnosis without exposing registrations or keys.
#[derive(thiserror::Error)]
#[error("overflow operation failed at {location}")]
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

pub(crate) trait ResultExt<T> {
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
