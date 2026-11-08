//! Runtime composition and common service boundary implementations.

use std::sync::Arc;

use chrono::Datelike;
use ciproxy_store_interface::billing::state::{BillingState, Selection, Tier};
use ciproxy_store_interface::billing::store::{BillingRecord, BillingSession, BillingStore};
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::alerts::BillingAlerts;
use crate::catalog::{CheckoutReview, monthly_total};
use crate::clock::Clock;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::api::StripeApi;
use crate::stripe::wire::types::Subscription;
use crate::webhook::SigningSecret;

/// Trusted dashboard redirects, configured by runtime composition.
#[derive(Clone, Debug)]
pub struct Redirects {
    /// Checkout success URL, including Stripe's session placeholder.
    pub success_url: String,
    /// Checkout cancel URL.
    pub cancel_url: String,
    /// Customer portal return URL.
    pub portal_return_url: String,
}

/// Billing runtime with dyn persistence, Stripe, and clock boundaries.
pub struct BillingService {
    pub(super) store: Arc<dyn BillingStore>,
    pub(super) stripe: Arc<dyn StripeApi>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) alerts: Arc<dyn BillingAlerts>,
    pub(super) signing_secret: SigningSecret,
    pub(super) redirects: Redirects,
}

impl BillingService {
    /// Construct the runtime from trusted composition dependencies.
    pub fn new(
        store: Arc<dyn BillingStore>,
        stripe: Arc<dyn StripeApi>,
        clock: Arc<dyn Clock>,
        alerts: Arc<dyn BillingAlerts>,
        signing_secret: SigningSecret,
        redirects: Redirects,
    ) -> Self {
        Self {
            store,
            stripe,
            clock,
            alerts,
            signing_secret,
            redirects,
        }
    }

    pub(super) fn lock(&self, workspace: Uuid) -> Result<Box<dyn BillingSession>> {
        self.store
            .try_lock_billing(workspace)
            .internal()?
            .ok_or(Error::Busy)
    }

    pub(super) fn current_subscription(
        &self,
        session: &mut dyn BillingSession,
        state: &BillingState,
    ) -> Result<Subscription> {
        let id = state
            .subscription_id
            .as_deref()
            .ok_or(Error::SubscriptionRequired)?;
        self.read_subscription(session, state, id)
    }

    pub(super) fn status(&self, access: BillingAccess) -> Result<BillingRecord> {
        self.store.billing(access.workspace_id).internal()
    }

    pub(super) fn review(
        &self,
        access: BillingAccess,
        selection: Selection,
    ) -> Result<CheckoutReview> {
        access.require_payer()?;
        if selection.tier == Tier::Free {
            return Err(Error::PaidTierRequired);
        }
        let cents = monthly_total(selection)?;
        Ok(CheckoutReview {
            monthly_cents: cents,
            today_cents: cents,
            billing_day: self.clock.now().day(),
        })
    }

    pub(super) fn portal(&self, access: BillingAccess) -> Result<String> {
        let workspace = access.require_payer()?;
        let record = self.store.billing(workspace).internal()?;
        if record.state.tier == Tier::Free {
            return Err(Error::SubscriptionRequired);
        }
        let customer = record
            .state
            .customer_id
            .ok_or(Error::SubscriptionRequired)?;
        self.stripe
            .portal(&customer, &self.redirects.portal_return_url)
    }
}
