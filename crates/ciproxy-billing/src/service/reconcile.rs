//! Public dyn service implementation delegates to each coherent workflow.

use ciproxy_store_interface::billing::state::{AddOns, Selection};
use ciproxy_store_interface::billing::store::BillingRecord;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::catalog::CheckoutReview;
use crate::error::Result;

use super::api::BillingApi;
use super::runtime::BillingService;
use super::view::{ChangeOutcome, CheckoutOutcome, EventOutcome};

impl BillingApi for BillingService {
    fn status(&self, access: BillingAccess) -> Result<BillingRecord> {
        self.status(access)
    }
    fn review(&self, access: BillingAccess, selection: Selection) -> Result<CheckoutReview> {
        self.review(access, selection)
    }
    fn open_checkout(
        &self,
        access: BillingAccess,
        selection: Selection,
    ) -> Result<CheckoutOutcome> {
        self.open_checkout(access, selection)
    }
    fn reconcile_checkout(&self, access: BillingAccess, id: &str) -> Result<CheckoutOutcome> {
        self.reconcile_checkout(access, id)
    }
    fn change(&self, access: BillingAccess, selection: Selection) -> Result<ChangeOutcome> {
        self.change(access, selection)
    }
    fn change_add_ons(&self, access: BillingAccess, add_ons: AddOns) -> Result<ChangeOutcome> {
        self.change_add_ons(access, add_ons)
    }
    fn move_to_free(&self, access: BillingAccess) -> Result<()> {
        self.move_to_free(access)
    }
    fn portal(&self, access: BillingAccess) -> Result<String> {
        self.portal(access)
    }
    fn webhook(&self, signature: &str, raw: &[u8]) -> Result<EventOutcome> {
        self.webhook(signature, raw)
    }
    fn reconcile_removal(&self, workspace: Uuid) -> Result<()> {
        self.reconcile_removal(workspace)
    }
}
