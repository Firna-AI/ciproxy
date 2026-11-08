//! Workspace billing pages and payer-only command contracts.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use super::money::Cents;
use super::pages::PageBanner;

/// The manually selected product plan.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanTier {
    /// The initial plan with no card.
    Free,
    /// The first monthly paid plan.
    Team,
    /// The larger monthly paid plan.
    Business,
}

/// Customer-selected extra capacity.
#[derive(Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct PlanAddOns {
    /// Blocks of ten thousand starts.
    pub start_blocks: u32,
    /// Additional own machines.
    pub machines: u32,
}

/// A plan choice, including its manually selected add-ons.
#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
pub struct PlanSelection {
    /// Selected plan.
    pub tier: PlanTier,
    /// Selected add-ons.
    pub add_ons: PlanAddOns,
}

/// Published capacity and the monthly price of a choice.
#[derive(Serialize, ToSchema)]
pub struct PlanOffer {
    /// Plan and add-on quantities.
    pub selection: PlanSelection,
    /// Included starts as a decimal string.
    pub included_starts: String,
    /// Included registered machines as a decimal string.
    pub included_machines: String,
    /// Monthly integer US cents.
    pub monthly_cents: Cents,
}

/// Confirmed billing status without Stripe customer or subscription identifiers.
#[derive(Serialize, ToSchema)]
pub struct PlanStatus {
    /// Confirmed plan and add-ons.
    pub selection: PlanSelection,
    /// Current subscription lifecycle.
    pub subscription: SubscriptionStatus,
    /// Inclusive UTC period start.
    pub period_start: DateTime<Utc>,
    /// Exclusive UTC period end.
    pub period_end: DateTime<Utc>,
    /// Scheduled end of paid coverage.
    pub cancel_at: Option<DateTime<Utc>>,
    /// A durable payment is open or awaiting confirmation.
    pub payment_pending: bool,
    /// A saved billing command is in progress, including cancellation.
    pub operation_in_progress: bool,
    /// Current caller may pay and open invoices.
    pub can_pay: bool,
}

/// Public subscription lifecycle.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    /// No subscription has started.
    None,
    /// Paid coverage continues.
    Active,
    /// Paid coverage continues while Stripe retries.
    PaymentFailed,
    /// Paid coverage ended.
    Ended,
}

/// Plan and usage page for all workspace roles.
#[derive(Serialize, ToSchema)]
pub struct PlanUsagePage {
    /// Confirmed subscription and payer capabilities.
    pub status: PlanStatus,
    /// Current tier's included capacity and monthly total.
    pub current: PlanOffer,
    /// Runner-taken starts in this period.
    pub starts: String,
    /// Current registrations, including offline and draining machines.
    pub machines: String,
    /// Current linked organizations, including inactive ones.
    pub organizations: String,
    /// Forecast starts as an exact decimal string.
    pub expected_starts: String,
    /// Starts above capacity as a decimal string.
    pub starts_over: String,
    /// Machines above capacity as a decimal string.
    pub machines_over: String,
    /// Organizations above Free capacity as a decimal string.
    pub organizations_over: String,
    /// Projected UTC date strictly above starts capacity.
    pub crossing_day: Option<NaiveDate>,
    /// Cheapest manually selectable paid choice.
    pub offer: PlanOffer,
    /// One page banner after priority rules.
    pub banner: Option<PageBanner>,
}

/// One plan card, including launch availability of its features.
#[derive(Serialize, ToSchema)]
pub struct PlanCard {
    /// Published base plan capacity and price.
    pub plan: PlanOffer,
    /// Current confirmed plan.
    pub current: bool,
    /// Offer plan when it differs from the current plan.
    pub covers_use: bool,
    /// Selection can proceed for this payer now.
    pub can_choose: bool,
    /// Finished-job window in days.
    pub history_days: u32,
    /// Linked-organization capacity, absent for unlimited.
    pub organizations: Option<u32>,
    /// Product features and availability.
    pub features: Vec<PlanFeature>,
    /// Initial checkout quantities for this card.
    pub initial_add_ons: PlanAddOns,
}

/// One feature with an explicit launch state.
#[derive(Serialize, ToSchema)]
pub struct PlanFeature {
    /// Customer-facing feature name.
    pub name: String,
    /// True only for a feature not yet available.
    pub coming_later: bool,
}

/// Plans page with confirmed status and an advisory offer.
#[derive(Serialize, ToSchema)]
pub struct PlansPage {
    /// Current use, offer, and the one page banner.
    pub usage: PlanUsagePage,
    /// Published Free, Team, and Business cards.
    pub plans: Vec<PlanCard>,
}

/// Checkout review before any payment begins.
#[derive(Serialize, ToSchema)]
pub struct CheckoutReviewPage {
    /// Authoritative paid review, absent for an initial Free Checkout.
    pub quote: Option<PaidQuoteReference>,
    /// Selected plan, capacities, and monthly total.
    pub choice: PlanOffer,
    /// Initial Checkout payment in integer US cents.
    pub today_cents: Cents,
    /// UTC day of each later monthly charge.
    pub billing_day: u32,
    /// Paid changes use Stripe's prorated saved-card charge.
    pub prorated: bool,
    /// Current period end for a paid change.
    pub period_end: DateTime<Utc>,
    /// One page banner.
    pub banner: Option<PageBanner>,
}

/// The server-owned quote which permits one paid change.
#[derive(Serialize, ToSchema)]
pub struct PaidQuoteReference {
    /// Opaque workspace-owned quote identifier.
    pub id: Uuid,
    /// Exclusive deadline for starting its payment.
    pub expires_at: DateTime<Utc>,
}

/// A paid change uses only the quote selected during review.
#[derive(Deserialize, ToSchema)]
pub struct PaidChangePayment {
    /// Authoritative quote identity. Selection and amounts stay on the server.
    pub quote_id: Uuid,
}

/// A Stripe Checkout return supplied only to the authorized payer.
#[derive(Deserialize, ToSchema)]
pub struct CheckoutReturn {
    /// Exact session returned by Stripe.
    pub session_id: String,
}

/// Recoverable Checkout and paid-change outcomes.
#[derive(Serialize, ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PaymentOutcome {
    /// Continue an existing hosted payment.
    Open {
        /// Stripe-hosted payment URL.
        url: String,
    },
    /// Keep polling without opening another payment.
    Confirming,
    /// Confirmed payment or paid change.
    Confirmed,
    /// Checkout ended without changing the plan.
    Unfinished,
    /// Paid change kept the old plan until payment succeeds.
    PaymentRequired {
        /// Hosted invoice or bank-confirmation page when Stripe provides one.
        payment_url: Option<String>,
    },
}

/// Checkout return page state, including the current shared banner.
#[derive(Serialize, ToSchema)]
pub struct CheckoutReturnPage {
    /// Stripe reconciliation result for this exact payer's session.
    pub outcome: PaymentOutcome,
    /// One banner from the latest confirmed workspace state.
    pub banner: Option<PageBanner>,
}

/// One Stripe portal redirect for invoices, card, or cancellation.
#[derive(Serialize, ToSchema)]
pub struct PortalLink {
    /// Stripe-hosted customer portal URL.
    pub url: String,
}

/// Checkout selection that starts with offer quantities for its suggested tier.
#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CheckoutReviewQuery {
    /// Desired plan, defaulting to the current offer.
    pub tier: Option<PlanTier>,
    /// Explicit start blocks, defaulting to the offer for the suggested plan.
    pub start_blocks: Option<u32>,
    /// Explicit machines, defaulting to the offer for the suggested plan.
    pub machines: Option<u32>,
}
