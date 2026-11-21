//! Billing outcomes consumed by the dashboard API.

use ciproxy_store_interface::billing::state::BillingState;

/// Checkout progress, suitable for return-page polling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutOutcome {
    /// Continue the same hosted Checkout.
    Open {
        /// The Stripe URL of the existing session.
        url: String,
    },
    /// Keep polling. A second payment must not open.
    Confirming,
    /// Stripe confirmed the payment and the local paid state.
    Confirmed(BillingState),
    /// Checkout expired. The tier stayed unchanged and jobs keep running.
    Unfinished,
}

/// The result of a paid tier or add-on change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeOutcome {
    /// Stripe applied the requested change.
    Applied(BillingState),
    /// Stripe kept the old tier while payment needs attention.
    PaymentRequired {
        /// The hosted invoice page for payment or bank confirmation.
        payment_url: Option<String>,
    },
}

/// A verified event's durable application result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventOutcome {
    /// The event and state committed atomically.
    Applied,
    /// A previously committed receipt prevents repeated application.
    Duplicate,
    /// This event type does not affect workspace billing.
    Ignored,
}
