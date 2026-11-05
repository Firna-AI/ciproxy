//! Stored subscription identities route known events before provider reads.

use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::webhook::{Event, Payload};

use super::runtime::BillingService;

pub(super) struct EventRoute {
    pub(super) workspace: Uuid,
    pub(super) target: String,
    pub(super) checkout: bool,
}

impl BillingService {
    pub(super) fn route_event(&self, event: &Event) -> Result<Option<EventRoute>> {
        if let Some(id) = event.subscription_id()
            && let Some(workspace) = self
                .store
                .billing_workspace_for_subscription(id)
                .internal()?
        {
            return Ok(Some(EventRoute {
                workspace,
                target: id.into(),
                checkout: false,
            }));
        }
        let (workspace, target, checkout) = match &event.payload {
            Payload::Checkout(object) => {
                let current = self.stripe.checkout_session(&object.object.id)?;
                (
                    current.metadata.workspace_id.ok_or(Error::Ownership)?,
                    current.id,
                    true,
                )
            }
            Payload::Subscription(object) => {
                let current = self.stripe.subscription(&object.object.id)?;
                (
                    current.metadata.workspace_id.ok_or(Error::Ownership)?,
                    current.id,
                    false,
                )
            }
            Payload::Invoice(object) | Payload::InvoiceFailed(object) => {
                let Some(route) = self.invoice_target(&object.object.id)? else {
                    return Ok(None);
                };
                return Ok(Some(route));
            }
            Payload::InvoiceUpdated(update) => {
                return self.invoice_target(&update.object.id);
            }
            Payload::Ignore => return Ok(None),
        };
        Ok(Some(EventRoute {
            workspace,
            target,
            checkout,
        }))
    }

    fn invoice_target(&self, invoice_id: &str) -> Result<Option<EventRoute>> {
        let invoice = self.stripe.invoice(invoice_id)?;
        let Some(id) = invoice.subscription_id() else {
            return Ok(None);
        };
        let current = self.stripe.subscription(id)?;
        Ok(Some(EventRoute {
            workspace: current.metadata.workspace_id.ok_or(Error::Ownership)?,
            target: id.into(),
            checkout: false,
        }))
    }
}
