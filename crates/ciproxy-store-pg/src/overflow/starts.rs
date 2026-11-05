//! Overflow session methods on the live advisory-lock connection.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::connection::ConnectionGrant;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartRecord};
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSession, OverflowSnapshot};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

use super::{Session, accounts, authority, completion, connections, deliveries, dispatch, rows};

impl OverflowSession for Session {
    fn snapshot(&mut self) -> Result<OverflowSnapshot> {
        rows::snapshot(&mut self.connection, self.workspace_id)
    }

    fn startup_account(&mut self, delivery_id: Uuid) -> Result<Option<AccountRecord>> {
        authority::startup_account(&mut self.connection, self.workspace_id, delivery_id)
    }

    fn begin_connection(&mut self, key_id: Uuid) -> Result<Option<ConnectionGrant>> {
        connections::begin(&mut self.connection, self.workspace_id, key_id)
    }

    fn complete_connection(
        &mut self,
        grant: ConnectionGrant,
        account: &AccountRecord,
        limit: i64,
    ) -> Result<bool> {
        accounts::connect(
            &mut self.connection,
            self.workspace_id,
            grant,
            account,
            limit,
        )
    }

    fn disconnect(&mut self, at: DateTime<Utc>) -> Result<()> {
        accounts::retire(&mut self.connection, self.workspace_id, at)
    }

    fn save_account(&mut self, account: &AccountRecord) -> Result<()> {
        accounts::save(&mut self.connection, self.workspace_id, account)
    }

    fn save_refusal(&mut self, account: &AccountRecord, record: &StartRecord) -> Result<()> {
        transaction(&mut self.connection, |connection| {
            accounts::save(connection, self.workspace_id, account)?;
            deliveries::save(connection, self.workspace_id, record)
        })
    }

    fn reserve_start(
        &mut self,
        record: &StartRecord,
        launch: &EncryptedLaunch,
        token_hash: &TokenHash,
        budget: &ReservationBudget,
    ) -> Result<ReservationOutcome> {
        deliveries::reserve(
            &mut self.connection,
            self.workspace_id,
            record,
            launch,
            token_hash,
            budget,
        )
    }

    fn encrypted_launch(&mut self, delivery_id: Uuid) -> Result<Option<EncryptedLaunch>> {
        Ok(overflow_starts::table
            .filter(overflow_starts::workspace_id.eq(self.workspace_id))
            .find(delivery_id)
            .select(overflow_starts::launch_ciphertext)
            .first::<Option<Vec<u8>>>(&mut self.connection)
            .optional()
            .store()?
            .flatten()
            .map(EncryptedLaunch))
    }

    fn begin_dispatch(
        &mut self,
        record: &StartRecord,
        budget: &ReservationBudget,
    ) -> Result<ReservationOutcome> {
        dispatch::begin(&mut self.connection, self.workspace_id, record, budget)
    }

    fn save_start(&mut self, record: &StartRecord) -> Result<()> {
        deliveries::save(&mut self.connection, self.workspace_id, record)
    }

    fn accept_end(
        &mut self,
        hash: &TokenHash,
        runner_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<Option<StartRecord>> {
        completion::accept(&mut self.connection, self.workspace_id, hash, runner_id, at)
    }

    fn collect(&mut self, before: DateTime<Utc>) -> Result<()> {
        diesel::delete(
            overflow_starts::table
                .filter(overflow_starts::workspace_id.eq(self.workspace_id))
                .filter(overflow_starts::ends_at.lt(before)),
        )
        .execute(&mut self.connection)
        .store()?;
        crate::boat::erase_unused_keys(&mut self.connection, self.workspace_id).store()?;
        Ok(())
    }
}
