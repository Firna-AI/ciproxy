//! Raw-body Stripe signature verification and typed event routing.

use std::fmt;

use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::error::{Error, Result, ResultExt};

/// The webhook signing secret, redacted from all diagnostic output.
pub struct SigningSecret(String);

impl SigningSecret {
    /// Accept a signing secret from runtime secret configuration.
    pub fn new(secret: String) -> Self {
        Self(secret)
    }

    /// Verify the exact raw request bytes and Stripe's five-minute tolerance.
    pub fn verify(&self, header: &str, raw: &[u8], now: i64) -> Result<()> {
        let timestamp = header
            .split(',')
            .find_map(|part| part.trim().strip_prefix("t="))
            .ok_or(Error::InvalidSignature)?;
        let timestamp = match timestamp.parse::<i64>() {
            Ok(value) => value,
            Err(_) => return Err(Error::InvalidSignature),
        };
        if now.abs_diff(timestamp) > 300 {
            return Err(Error::InvalidSignature);
        }
        let mut mac = match Hmac::<Sha256>::new_from_slice(self.0.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return Err(Error::InvalidSignature),
        };
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(raw);
        for signature in header
            .split(',')
            .filter_map(|part| part.trim().strip_prefix("v1="))
        {
            if let Ok(bytes) = hex::decode(signature)
                && mac.clone().verify_slice(&bytes).is_ok()
            {
                return Ok(());
            }
        }
        Err(Error::InvalidSignature)
    }
}

impl fmt::Debug for SigningSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SigningSecret([redacted])")
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Event {
    pub(crate) id: String,
    pub(crate) created: i64,
    #[serde(flatten)]
    pub(crate) payload: Payload,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub(crate) enum Payload {
    #[serde(
        rename = "checkout.session.completed",
        alias = "checkout.session.async_payment_succeeded",
        alias = "checkout.session.async_payment_failed",
        alias = "checkout.session.expired"
    )]
    Checkout(Object),
    #[serde(
        rename = "customer.subscription.created",
        alias = "customer.subscription.updated",
        alias = "customer.subscription.deleted",
        alias = "customer.subscription.pending_update_applied",
        alias = "customer.subscription.pending_update_expired"
    )]
    Subscription(Object),
    #[serde(rename = "invoice.payment_failed")]
    InvoiceFailed(Object),
    #[serde(
        rename = "invoice.paid",
        alias = "invoice.payment_action_required",
        alias = "invoice.updated",
        alias = "invoice.marked_uncollectible"
    )]
    Invoice(Object),
    #[serde(other)]
    Ignore,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Object {
    pub(crate) object: ObjectId,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ObjectId {
    pub(crate) id: String,
}

pub(crate) fn parse(raw: &[u8]) -> Result<Event> {
    serde_json::from_slice(raw).internal()
}
