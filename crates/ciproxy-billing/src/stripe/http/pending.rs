//! Complete pending-item reads and idempotent retirement of original items.

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::pending::InvoiceItem;
use crate::stripe::wire::types::List;

use super::client::HttpStripe;

impl HttpStripe {
    pub(super) fn list_pending_items(&self, customer: &str) -> Result<Vec<InvoiceItem>> {
        let mut result = Vec::new();
        let mut after: Option<String> = None;
        loop {
            let mut query = vec![
                ("customer".into(), customer.into()),
                ("pending".into(), "true".into()),
                ("limit".into(), "100".into()),
            ];
            if let Some(id) = &after {
                query.push(("starting_after".into(), id.clone()));
            }
            let page = self
                .get("/v1/invoiceitems", &query)?
                .json::<List<InvoiceItem>>()
                .internal()?;
            if page
                .data
                .iter()
                .any(|item| item.customer != customer || item.invoice.is_some())
            {
                return Err(Error::Ownership);
            }
            if page.has_more && page.data.is_empty() {
                return Err(Error::StripeState);
            }
            after = page.data.last().map(|item| item.id.clone());
            result.extend(page.data);
            if !page.has_more {
                return Ok(result);
            }
        }
    }

    pub(super) fn read_invoice_item(&self, id: &str) -> Result<Option<InvoiceItem>> {
        match self.get(&format!("/v1/invoiceitems/{id}"), &[]) {
            Ok(response) => Ok(Some(response.json::<InvoiceItem>().internal()?)),
            Err(Error::StripeHttp { status: 404 }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(super) fn remove_invoice_item(&self, id: &str) -> Result<()> {
        match self.delete(&format!("/v1/invoiceitems/{id}"), &[], None) {
            Ok(_) | Err(Error::StripeRejected { status: 404 }) => Ok(()),
            Err(Error::StripeValidationRejected { proof }) if proof.status == 404 => Ok(()),
            Err(error) => Err(error),
        }
    }
}
