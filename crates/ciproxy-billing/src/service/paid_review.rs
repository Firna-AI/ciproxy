//! Authoritative paid-plan quotes under the same workspace billing lock.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use unimock::unimock;

use crate::access::BillingAccess;
use crate::catalog::CheckoutReview;
use crate::error::{Error, Result, ResultExt};

use super::runtime::BillingService;
use super::transition::{selection, terminal};

/// Stripe invoice previews for an exact paid selection and observation time.
#[unimock(api = PaidReviewMock)]
pub trait PaidReview: Send + Sync {
    /// Quote the monthly amount, today's charge, and recurring UTC anchor day.
    fn review(
        &self,
        access: BillingAccess,
        selection: Selection,
        at: DateTime<Utc>,
    ) -> Result<CheckoutReview>;
}

impl PaidReview for BillingService {
    fn review(
        &self,
        access: BillingAccess,
        wanted: Selection,
        at: DateTime<Utc>,
    ) -> Result<CheckoutReview> {
        let workspace = access.require_payer()?;
        if wanted.tier == Tier::Free {
            return Err(Error::PaidTierRequired);
        }
        let mut session = self.lock(workspace)?;
        let record = session.record().internal()?;
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
        self.stripe.preview(&current, wanted, at, upgrade)
    }
}
