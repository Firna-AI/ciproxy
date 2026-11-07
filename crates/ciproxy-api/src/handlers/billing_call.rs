//! Blocking billing execution and safe typed HTTP error translation.

use std::sync::Arc;

use ciproxy_billing::service::api::BillingApi;

use crate::error::{Error, ErrorResultExt, Result};
use crate::pages::PageServices;

pub(super) async fn call<T: Send + 'static>(
    services: Arc<PageServices>,
    action: impl FnOnce(&dyn BillingApi) -> ciproxy_billing::error::Result<T> + Send + 'static,
) -> Result<T> {
    result(
        tokio::task::spawn_blocking(move || action(services.billing.as_ref()))
            .await
            .internal()?,
    )
}

pub(super) fn result<T>(value: ciproxy_billing::error::Result<T>) -> Result<T> {
    match value {
        Ok(value) => Ok(value),
        Err(
            ciproxy_billing::error::Error::Forbidden | ciproxy_billing::error::Error::Ownership,
        ) => Err(Error::Forbidden),
        Err(
            ciproxy_billing::error::Error::Busy
            | ciproxy_billing::error::Error::PaymentConfirming
            | ciproxy_billing::error::Error::CurrentTier
            | ciproxy_billing::error::Error::SubscriptionRequired,
        ) => Err(Error::Conflict),
        Err(
            ciproxy_billing::error::Error::FreeAddOns
            | ciproxy_billing::error::Error::PaidTierRequired
            | ciproxy_billing::error::Error::InvalidSignature,
        ) => Err(Error::InvalidInput),
        Err(
            ciproxy_billing::error::Error::StripeHttp { .. }
            | ciproxy_billing::error::Error::StripeRejected { .. }
            | ciproxy_billing::error::Error::StripeUnavailable
            | ciproxy_billing::error::Error::StripeState,
        ) => Err(Error::Unavailable),
        Err(source) => Err(Error::internal(source)),
    }
}
