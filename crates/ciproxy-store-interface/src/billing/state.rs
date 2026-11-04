//! Billing values shared by services, metering, and store adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The manually selected workspace tier.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// The initial tier, requiring neither a customer nor a card.
    #[default]
    Free,
    /// The monthly Team tier.
    Team,
    /// The monthly Business tier.
    Business,
}

/// Quantities the customer selects, never inferred from use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddOns {
    /// Blocks of 10,000 starts.
    pub start_blocks: u32,
    /// Additional registered machines.
    pub machines: u32,
}

/// A requested paid tier and monthly quantities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    /// The chosen tier.
    pub tier: Tier,
    /// The chosen add-ons.
    pub add_ons: AddOns,
}

/// The state shown to all workspace members.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionState {
    /// No paid subscription has started.
    #[default]
    None,
    /// Stripe continues paid coverage.
    Active,
    /// Stripe retries a failed renewal while paid coverage stays.
    PaymentFailed,
    /// The paid subscription ended.
    Ended,
}

/// The shared billing read model, with UTC period boundaries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BillingState {
    /// The workspace that owns the billing data.
    pub workspace_id: Uuid,
    /// The confirmed tier.
    pub tier: Tier,
    /// The confirmed add-ons.
    pub add_ons: AddOns,
    /// The Stripe customer, absent for a new Free workspace.
    pub customer_id: Option<String>,
    /// The last confirmed Stripe subscription.
    pub subscription_id: Option<String>,
    /// The confirmed subscription state.
    pub status: SubscriptionState,
    /// The anchor of the current run of Free periods.
    pub free_started_at: DateTime<Utc>,
    /// The inclusive Stripe paid-period start.
    pub paid_period_start: Option<DateTime<Utc>>,
    /// The exclusive Stripe paid-period end.
    pub paid_period_end: Option<DateTime<Utc>>,
    /// The date on which the current paid tier is scheduled to end.
    pub cancel_at: Option<DateTime<Utc>>,
    /// The most recent failed payment date, while Stripe retries.
    pub failed_at: Option<DateTime<Utc>>,
    /// The invoice associated with the current failed-payment date.
    pub failed_invoice_id: Option<String>,
    /// Stripe's next scheduled payment attempt, when available.
    pub next_retry_at: Option<DateTime<Utc>>,
    /// The last instant of the ended-tier banner, exclusive.
    pub ended_banner_until: Option<DateTime<Utc>>,
}

impl BillingState {
    /// Initialize Free at workspace creation, without contacting Stripe.
    pub fn free(workspace_id: Uuid, at: DateTime<Utc>) -> Self {
        Self {
            workspace_id,
            tier: Tier::Free,
            add_ons: AddOns::default(),
            customer_id: None,
            subscription_id: None,
            status: SubscriptionState::None,
            free_started_at: at,
            paid_period_start: None,
            paid_period_end: None,
            cancel_at: None,
            failed_at: None,
            failed_invoice_id: None,
            next_retry_at: None,
            ended_banner_until: None,
        }
    }
}
