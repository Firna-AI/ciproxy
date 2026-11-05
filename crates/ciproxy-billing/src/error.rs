//! Typed recoverable billing failures with redacted internal diagnostics.

use std::error;
use std::fmt;
use std::panic::Location;

/// A billing operation failure which callers can handle.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The caller has no billing authority.
    #[error("[billing/access] Admin or Billing role required")]
    Forbidden,
    /// Another command or event owns the workspace session.
    #[error("[billing/service] another billing operation is in progress")]
    Busy,
    /// A current Checkout or unpaid change must finish first.
    #[error("[billing/service] payment confirmation is in progress")]
    PaymentConfirming,
    /// Free cannot buy add-ons.
    #[error("[billing/catalog] Free has no add-ons")]
    FreeAddOns,
    /// Checkout and paid item changes need a paid tier.
    #[error("[billing/catalog] select Team or Business")]
    PaidTierRequired,
    /// The requested tier is already the current tier.
    #[error("[billing/service] tier is already current")]
    CurrentTier,
    /// Paid billing requires an existing paid subscription.
    #[error("[billing/service] paid subscription required")]
    SubscriptionRequired,
    /// A signed event or return session belongs elsewhere.
    #[error("[billing/service] Stripe object does not belong to this workspace")]
    Ownership,
    /// Signature verification failed or the delivery is too old.
    #[error("[billing/webhook] invalid signature")]
    InvalidSignature,
    /// Stripe could not complete a request. Retry with the saved command.
    #[error("[billing/stripe] request failed with HTTP {status}")]
    StripeHttp {
        /// The HTTP status, without Stripe's response body.
        status: u16,
    },
    /// Stripe definitively rejected a mutation before it could apply.
    #[error("[billing/stripe] mutation rejected with HTTP {status}")]
    StripeRejected {
        /// The rejection status, without the provider response body.
        status: u16,
    },
    /// A network call failed before a definite outcome was known.
    #[error("[billing/stripe] request outcome is unknown, retry reconciliation")]
    StripeUnavailable,
    /// The current Stripe object cannot safely supply billing state.
    #[error("[billing/stripe] current object is incomplete or unsupported")]
    StripeState,
    /// A provider or persistence implementation failed internally.
    #[error("[billing/internal] {0}")]
    Internal(#[from] InternalError),
}

/// The result of a billing operation.
pub type Result<T> = std::result::Result<T, Error>;

/// An original failure whose Display and Debug output hide all values.
#[derive(thiserror::Error)]
#[error("billing operation failed at {location}")]
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
    /// Preserve the source without exposing payment or credential data.
    #[track_caller]
    pub fn internal(source: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self::Internal(InternalError {
            source: source.into(),
            location: Location::caller(),
        })
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
            Err(source) => Err(Error::internal(source)),
        }
    }
}
