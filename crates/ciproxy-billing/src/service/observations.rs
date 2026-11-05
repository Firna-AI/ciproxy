//! Owned locked reads persist every collection observation before returning.

use ciproxy_store_interface::billing::state::{BillingState, Tier};
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::Invoice;
use crate::stripe::wire::types::Subscription;

use super::collection_decisions::{CollectionObservation, merge};
use super::runtime::BillingService;
use super::transition::confirmed;

impl BillingService {
    pub(super) fn read_subscription(
        &self,
        session: &mut dyn BillingSession,
        old: &BillingState,
        id: &str,
    ) -> Result<Subscription> {
        let current = self.stripe.subscription(id)?;
        if current.id != id {
            return Err(Error::Ownership);
        }
        self.observe_subscription(session, old, &current)?;
        Ok(current)
    }

    pub(super) fn observe_subscription(
        &self,
        session: &mut dyn BillingSession,
        old: &BillingState,
        current: &Subscription,
    ) -> Result<()> {
        if old.tier != Tier::Free && old.subscription_id.as_ref() != Some(&current.id) {
            return Err(Error::Ownership);
        }
        confirmed(old, current, self.clock.now(), None)?;
        for mut renewal in session.renewals(&current.id).internal()? {
            let previous = renewal.collection.clone();
            merge(
                &mut renewal.collection,
                CollectionObservation::Subscription {
                    current,
                    invoice_id: &renewal.invoice_id,
                },
            );
            if previous != renewal.collection {
                session.save_renewal(&renewal).internal()?;
            }
        }
        Ok(())
    }

    pub(super) fn owned_invoice(
        &self,
        session: &mut dyn BillingSession,
        current: &Subscription,
        id: &str,
    ) -> Result<Invoice> {
        let record = self.stripe.invoice(id)?;
        if record.invoice.id != id
            || record.customer != current.customer
            || record.subscription_id() != Some(current.id.as_str())
        {
            return Err(Error::Ownership);
        }
        if let Some(mut renewal) = session
            .renewals(&current.id)
            .internal()?
            .into_iter()
            .find(|renewal| renewal.invoice_id == id)
        {
            let previous = renewal.collection.clone();
            merge(
                &mut renewal.collection,
                CollectionObservation::Invoice(&record.invoice),
            );
            if previous != renewal.collection {
                session.save_renewal(&renewal).internal()?;
            }
        }
        Ok(record.invoice)
    }
}
