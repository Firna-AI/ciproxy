//! Immutable invoice correlation and discovered signed balance restoration.

use std::collections::BTreeMap;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::RetrievedInvoice;
use crate::stripe::wire::types::List;

use super::client::HttpStripe;

#[derive(Deserialize)]
struct BalanceTransaction {
    id: String,
    customer: String,
    amount: i64,
    currency: String,
    metadata: Option<BalanceMetadata>,
}

#[derive(Deserialize)]
struct BalanceMetadata {
    ciproxy_payment_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct CustomerBalance {
    id: String,
    balance: i64,
    invoice_credit_balance: Option<BTreeMap<String, i64>>,
}

impl HttpStripe {
    pub(super) fn read_customer_balance(&self, customer: &str) -> Result<i64> {
        let current = self
            .get(&format!("/v1/customers/{customer}"), &[])?
            .json::<CustomerBalance>()
            .internal()?;
        if current.id != customer {
            return Err(Error::Ownership);
        }
        Ok(match current.invoice_credit_balance {
            Some(balances) => balances.get("usd").copied().unwrap_or(0),
            None => current.balance,
        })
    }

    pub(super) fn stamp_payment(
        &self,
        subscription: &str,
        quote: Uuid,
        command: Uuid,
    ) -> Result<()> {
        self.post(
            &format!("/v1/subscriptions/{subscription}"),
            &[
                ("metadata[ciproxy_quote_id]".into(), quote.to_string()),
                ("metadata[ciproxy_payment_id]".into(), command.to_string()),
            ],
            Some(&format!("ciproxy:quote-marker:{command}")),
        )?;
        Ok(())
    }

    pub(super) fn list_payment_invoices(
        &self,
        customer: &str,
        subscription: &str,
    ) -> Result<Vec<RetrievedInvoice>> {
        let mut result = Vec::new();
        let mut after: Option<String> = None;
        loop {
            let mut query = vec![
                ("customer".into(), customer.into()),
                ("subscription".into(), subscription.into()),
                ("limit".into(), "100".into()),
            ];
            if let Some(id) = &after {
                query.push(("starting_after".into(), id.clone()));
            }
            let page = self
                .get("/v1/invoices", &query)?
                .json::<List<RetrievedInvoice>>()
                .internal()?;
            if page.data.iter().any(|invoice| {
                invoice.customer != customer || invoice.subscription_id() != Some(subscription)
            }) {
                return Err(Error::Ownership);
            }
            if page.has_more && page.data.is_empty() {
                return Err(Error::StripeState);
            }
            after = page.data.last().map(|invoice| invoice.invoice.id.clone());
            result.extend(page.data);
            if !page.has_more {
                return Ok(result);
            }
        }
    }

    pub(super) fn restore_customer_balance(
        &self,
        customer: &str,
        command: Uuid,
        amount: i64,
    ) -> Result<String> {
        let path = format!("/v1/customers/{customer}/balance_transactions");
        let mut after: Option<String> = None;
        loop {
            let mut query = vec![("limit".into(), "100".into())];
            if let Some(id) = &after {
                query.push(("starting_after".into(), id.clone()));
            }
            let page = self
                .get(&path, &query)?
                .json::<List<BalanceTransaction>>()
                .internal()?;
            for transaction in &page.data {
                if transaction
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.ciproxy_payment_id)
                    == Some(command)
                {
                    if transaction.customer != customer
                        || transaction.amount != amount
                        || transaction.currency != "usd"
                    {
                        return Err(Error::StripeState);
                    }
                    return Ok(transaction.id.clone());
                }
            }
            if page.has_more && page.data.is_empty() {
                return Err(Error::StripeState);
            }
            after = page.data.last().map(|transaction| transaction.id.clone());
            if !page.has_more {
                break;
            }
        }
        let response = self
            .post(
                &path,
                &[
                    ("amount".into(), amount.to_string()),
                    ("currency".into(), "usd".into()),
                    ("metadata[ciproxy_payment_id]".into(), command.to_string()),
                ],
                Some(&format!("ciproxy:credit-restore:{command}")),
            )?
            .json::<BalanceTransaction>()
            .internal()?;
        if response.customer != customer || response.amount != amount || response.currency != "usd"
        {
            return Err(Error::Ownership);
        }
        Ok(response.id)
    }
}
