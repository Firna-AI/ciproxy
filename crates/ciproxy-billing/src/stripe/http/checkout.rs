//! Checkout creation and lost-reply discovery before idempotent replay.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::forms;
use crate::stripe::wire::types::{CheckoutSession, List};

use super::client::HttpStripe;

impl HttpStripe {
    pub(super) fn find_checkout(
        &self,
        workspace_id: Uuid,
        command: &BillingCommand,
    ) -> Result<Option<CheckoutSession>> {
        let mut query = vec![
            (
                "created[gte]".into(),
                command.started_at.timestamp().to_string(),
            ),
            ("limit".into(), "100".into()),
        ];
        loop {
            let page = self
                .get("/v1/checkout/sessions", &query)?
                .json::<List<CheckoutSession>>()
                .internal()?;
            if let Some(session) = page.data.iter().find(|session| {
                session.metadata.workspace_id == Some(workspace_id)
                    && session.metadata.command_id == Some(command.id)
            }) {
                return Ok(Some(session.clone()));
            }
            if !page.has_more {
                return Ok(None);
            }
            let last = page.data.last().ok_or(Error::StripeState)?;
            query.retain(|(key, _)| key != "starting_after");
            query.push(("starting_after".into(), last.id.clone()));
        }
    }

    pub(super) fn create_checkout(
        &self,
        workspace_id: Uuid,
        customer_id: Option<&str>,
        command: &BillingCommand,
    ) -> Result<CheckoutSession> {
        if let Some(session) = self.find_checkout(workspace_id, command)? {
            return Ok(session);
        }
        let CommandKind::Checkout { selection, .. } = command.kind else {
            return Err(Error::StripeState);
        };
        let mut form = forms::checkout(workspace_id, command)?;
        for (index, (price, quantity)) in self.prices(selection)?.into_iter().enumerate() {
            form.push((format!("line_items[{index}][price]"), price));
            form.push((
                format!("line_items[{index}][quantity]"),
                quantity.to_string(),
            ));
        }
        if let Some(customer_id) = customer_id {
            form.push(("customer".into(), customer_id.into()));
        }
        let key = format!("ciproxy:checkout:{}", command.id);
        self.post("/v1/checkout/sessions", &form, Some(&key))?
            .json::<CheckoutSession>()
            .internal()
    }
}
