//! Idempotent Stripe catalog and restricted portal configuration provisioning.

use ciproxy_store_interface::billing::state::Selection;
use serde::Deserialize;

use crate::catalog::{PriceCode, line_items};
use crate::error::{Error, Result, ResultExt};
use crate::stripe::forms;
use crate::stripe::wire::types::{List, Price};

use super::client::HttpStripe;

#[derive(Deserialize)]
pub(super) struct Identifier {
    pub(super) id: String,
}

#[derive(Deserialize)]
struct CatalogPrice {
    id: String,
    currency: String,
    unit_amount: u64,
    recurring: Recurring,
    active: bool,
}

#[derive(Deserialize)]
struct Recurring {
    interval: String,
    interval_count: u32,
}

impl HttpStripe {
    pub(super) fn ensure_price(&self, code: PriceCode) -> Result<String> {
        let list = self
            .get(
                "/v1/prices",
                &[
                    ("lookup_keys[0]".into(), code.lookup_key().into()),
                    ("active".into(), "true".into()),
                ],
            )?
            .json::<List<CatalogPrice>>()
            .internal()?;
        if let Some(price) = list.data.first() {
            if price.currency != "usd"
                || price.unit_amount != code.cents()
                || price.recurring.interval != "month"
                || price.recurring.interval_count != 1
                || !price.active
            {
                return Err(Error::StripeState);
            }
            return Ok(price.id.clone());
        }
        let key = format!("ciproxy:catalog:v1:{}", code.lookup_key());
        let form = vec![
            ("currency".into(), "usd".into()),
            ("unit_amount".into(), code.cents().to_string()),
            ("recurring[interval]".into(), "month".into()),
            ("lookup_key".into(), code.lookup_key().into()),
            ("product_data[name]".into(), code.lookup_key().into()),
            ("tax_behavior".into(), "exclusive".into()),
        ];
        Ok(self
            .post("/v1/prices", &form, Some(&key))?
            .json::<Price>()
            .internal()?
            .id)
    }

    pub(super) fn prices(&self, selection: Selection) -> Result<Vec<(String, u32)>> {
        line_items(selection)?
            .into_iter()
            .map(|(code, quantity)| Ok((self.ensure_price(code)?, quantity)))
            .collect()
    }

    pub(super) fn portal_configuration(&self) -> Result<String> {
        Ok(self
            .post(
                "/v1/billing_portal/configurations",
                &forms::portal_configuration(),
                Some("ciproxy:portal:v1"),
            )?
            .json::<Identifier>()
            .internal()?
            .id)
    }
}
