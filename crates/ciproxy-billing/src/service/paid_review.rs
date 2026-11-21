//! Authoritative paid-plan quotes under the same workspace billing lock.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::billing::quotes::{QuotePayment, StoredQuote};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use ciproxy_store_interface::metering::history::Period;
use unimock::unimock;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::catalog::CheckoutReview;
use crate::error::{Error, Result, ResultExt};

use super::payment_work::money_held;
use super::quote_source::source;
use super::runtime::BillingService;
use super::transition::{period, selection, terminal};

/// Paid amounts and current period from one owned Stripe subscription observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaidQuote {
    /// Workspace-owned review identity accepted by the payment operation.
    pub id: Uuid,
    /// Exclusive deadline for starting the payment.
    pub expires_at: DateTime<Utc>,
    /// Previewed monthly total, today's charge, and recurring billing day.
    pub amounts: CheckoutReview,
    /// The current Stripe billing period used by this preview.
    pub period: Period,
}

/// Stripe invoice previews for an exact paid selection and observation time.
#[unimock(api = PaidReviewMock)]
pub trait PaidReview: Send + Sync {
    /// Quote the paid amounts and the current Stripe billing period together.
    fn review(
        &self,
        access: BillingAccess,
        selection: Selection,
        at: DateTime<Utc>,
    ) -> Result<PaidQuote>;
}

impl PaidReview for BillingService {
    fn review(
        &self,
        access: BillingAccess,
        wanted: Selection,
        at: DateTime<Utc>,
    ) -> Result<PaidQuote> {
        let workspace = access.require_payer()?;
        if wanted.tier == Tier::Free {
            return Err(Error::PaidTierRequired);
        }
        let mut session = self.lock(workspace)?;
        let record = session.record().internal()?;
        if session.payments().internal()?.iter().any(money_held) {
            return Err(Error::PaymentConfirming);
        }
        if record.command.is_some() || record.checkout.is_some() {
            return Err(Error::PaymentConfirming);
        }
        if record.state.tier == Tier::Free {
            return Err(Error::SubscriptionRequired);
        }
        let current = self.current_subscription(&mut *session, &record.state)?;
        if terminal(&current) {
            return Err(Error::SubscriptionRequired);
        }
        if current.pending_update.is_some() {
            return Err(Error::PaymentConfirming);
        }
        let confirmed = selection(&current)?;
        if confirmed.tier == wanted.tier {
            return Err(Error::CurrentTier);
        }
        let upgrade = confirmed.tier == Tier::Team && wanted.tier == Tier::Business;
        let at = DateTime::from_timestamp(at.timestamp(), 0).ok_or(Error::StripeState)?;
        let period = period(&current)?;
        if at < period.starts_at || at >= period.ends_at {
            return Err(Error::ReviewRequired);
        }
        let preview = self.stripe.preview(&current, wanted, at, upgrade)?;
        let id = self.clock.command_id();
        let expires_at = (at + Duration::minutes(30)).min(period.ends_at);
        session
            .save_quote(&StoredQuote {
                payment: QuotePayment {
                    id,
                    proration_at: at,
                    items: preview.items,
                    offset: preview.offset,
                    starting_balance_cents: Some(preview.starting_balance_cents),
                },
                selection: wanted,
                upgrade,
                source: source(&current)?,
                monthly_cents: preview.amounts.monthly_cents,
                today_cents: preview.amounts.today_cents,
                billing_day: preview.amounts.billing_day,
                expires_at,
                command_id: None,
            })
            .internal()?;
        Ok(PaidQuote {
            id,
            expires_at,
            period,
            amounts: preview.amounts,
        })
    }
}
