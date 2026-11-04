//! Private typed mappings for billing persistence.

use chrono::{DateTime, Utc};
use ciproxy_schema::billing::billing_accounts;
use ciproxy_store_interface::billing::commands::{BillingCommand, CheckoutProgress};
use ciproxy_store_interface::billing::state::BillingState;
use ciproxy_store_interface::billing::store::BillingRecord;
use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

#[derive(Queryable, Selectable)]
#[diesel(table_name = billing_accounts)]
pub(super) struct AccountRow {
    workspace_id: Uuid,
    lock_key: i64,
    created_at: DateTime<Utc>,
    state_json: Option<String>,
    command_json: Option<String>,
    checkout_json: Option<String>,
    subscription_id: Option<String>,
    removal_cancel_requested: bool,
}

impl AccountRow {
    pub(super) fn record(self) -> Result<BillingRecord> {
        let state = match self.state_json {
            Some(json) => serde_json::from_str::<BillingState>(&json).internal()?,
            None => BillingState::free(self.workspace_id, self.created_at),
        };
        let command = match self.command_json {
            Some(json) => Some(serde_json::from_str::<BillingCommand>(&json).internal()?),
            None => None,
        };
        let checkout = match self.checkout_json {
            Some(json) => Some(serde_json::from_str::<CheckoutProgress>(&json).internal()?),
            None => None,
        };
        let _ = (self.lock_key, self.subscription_id);
        Ok(BillingRecord {
            state,
            command,
            checkout,
            removal_cancel_requested: self.removal_cancel_requested,
        })
    }
}

pub(super) fn read(connection: &mut PgConnection, workspace_id: Uuid) -> Result<BillingRecord> {
    billing_accounts::table
        .find(workspace_id)
        .select(AccountRow::as_select())
        .first::<AccountRow>(connection)
        .store()?
        .record()
}
