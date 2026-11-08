//! Durable mutations through a live exclusive billing session.

use chrono::{DateTime, Utc};
use ciproxy_schema::billing::{billing_accounts, billing_events};
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::billing::commands::{BillingCommand, CheckoutProgress};
use ciproxy_store_interface::billing::renewals::Renewal;
use ciproxy_store_interface::billing::state::BillingState;
use ciproxy_store_interface::billing::store::{BillingRecord, BillingSession};
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::error::QueryResultExt;

use super::{renewals, rows};

pub(super) struct Session {
    pub(super) connection: PgConnection,
    pub(super) workspace_id: Uuid,
    pub(super) lock_key: i64,
}

impl Drop for Session {
    /// Release the lock before closing the dedicated connection.
    ///
    /// Drop cannot return unlock errors. Connection teardown remains the
    /// fallback when PostgreSQL cannot process the explicit release.
    fn drop(&mut self) {
        let _ = diesel::sql_query("SELECT pg_advisory_unlock($1)")
            .bind::<BigInt, _>(self.lock_key)
            .execute(&mut self.connection);
    }
}

impl BillingSession for Session {
    fn renewals(&mut self, subscription_id: &str) -> Result<Vec<Renewal>> {
        renewals::read(&mut self.connection, self.workspace_id, subscription_id)
    }

    fn save_renewal(&mut self, renewal: &Renewal) -> Result<()> {
        renewals::save(&mut self.connection, self.workspace_id, renewal)
    }

    fn record(&mut self) -> Result<BillingRecord> {
        rows::read(&mut self.connection, self.workspace_id)
    }

    fn save_command(&mut self, command: &BillingCommand) -> Result<()> {
        let json = serde_json::to_string(command).internal()?;
        diesel::update(billing_accounts::table.find(self.workspace_id))
            .set(billing_accounts::command_json.eq(json))
            .execute(&mut self.connection)
            .store()?;
        Ok(())
    }

    fn save_checkout(&mut self, checkout: &CheckoutProgress) -> Result<()> {
        let json = serde_json::to_string(checkout).internal()?;
        diesel::update(billing_accounts::table.find(self.workspace_id))
            .set(billing_accounts::checkout_json.eq(json))
            .execute(&mut self.connection)
            .store()?;
        Ok(())
    }

    fn event_applied(&mut self, event_id: &str) -> Result<bool> {
        Ok(billing_events::table
            .find(event_id)
            .select(billing_events::event_id)
            .first::<String>(&mut self.connection)
            .optional()
            .store()?
            .is_some())
    }

    fn commit(
        &mut self,
        state: Option<&BillingState>,
        event_id: Option<&str>,
        finish_command: bool,
        handled_removal: bool,
        at: DateTime<Utc>,
    ) -> Result<()> {
        if state.is_some_and(|state| state.workspace_id != self.workspace_id) {
            return Err(Error::InvalidReference);
        }
        let json = state.map(serde_json::to_string).transpose().internal()?;
        self.connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                workspaces::table
                    .find(self.workspace_id)
                    .select(workspaces::id)
                    .for_update()
                    .first::<Uuid>(connection)?;
                if let Some(event_id) = event_id {
                    let inserted = diesel::insert_into(billing_events::table)
                        .values((
                            billing_events::event_id.eq(event_id),
                            billing_events::workspace_id.eq(self.workspace_id),
                            billing_events::applied_at.eq(at),
                        ))
                        .on_conflict_do_nothing()
                        .execute(connection)?;
                    if inserted == 0 {
                        return Ok(());
                    }
                }
                if let (Some(json), Some(state)) = (&json, state) {
                    diesel::update(billing_accounts::table.find(self.workspace_id))
                        .set((
                            billing_accounts::state_json.eq(json),
                            billing_accounts::subscription_id.eq(&state.subscription_id),
                        ))
                        .execute(connection)?;
                }
                if finish_command {
                    diesel::update(billing_accounts::table.find(self.workspace_id))
                        .set((
                            billing_accounts::command_json.eq::<Option<String>>(None),
                            billing_accounts::checkout_json.eq::<Option<String>>(None),
                        ))
                        .execute(connection)?;
                }
                if handled_removal {
                    diesel::update(billing_accounts::table.find(self.workspace_id))
                        .set(billing_accounts::removal_cancel_requested.eq(false))
                        .execute(connection)?;
                }
                Ok(())
            })
            .store()
    }
}
