//! Gateway query composition with atomic credential and report changes.

use chrono::{DateTime, Duration, Utc};
use ciproxy_schema::machines::{machine_reports, machines};
use ciproxy_schema::tokens::tokens;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::gateway::{Contact, GatewayStore, MachineSnapshot, Registration};
use ciproxy_store_interface::machines::{Machine, MachineStore};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_contacts::{lock_machine, session, write_contact};
use crate::gateway_registration::{connect, lock_workspace, register};
use crate::gateway_transaction::transaction;
use crate::machines::remove_machine_records;
use crate::tokens::insert_token;

impl GatewayStore for PgStore {
    fn issue_install_token(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        hash: &TokenHash,
        at: DateTime<Utc>,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_workspace(connection, workspace_id)?;
            let deleting = workspaces::table
                .find(workspace_id)
                .select(workspaces::delete_after)
                .first::<Option<DateTime<Utc>>>(connection)
                .store()?;
            if deleting.is_some() {
                return Err(Error::NotFound);
            }
            insert_token(
                connection,
                &Token {
                    id,
                    workspace_id,
                    subject: TokenSubject::Install,
                    created_at: at,
                    expires_at: Some(at + Duration::hours(1)),
                    revoked_at: None,
                },
                hash,
            )
            .store()
        })
    }

    fn revoke_install_token(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let count = diesel::update(
            tokens::table
                .filter(tokens::workspace_id.eq(workspace_id))
                .filter(tokens::purpose.eq("install"))
                .find(id),
        )
        .set(tokens::revoked_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn register_machine(&self, registration: &Registration) -> Result<bool> {
        transaction(&mut *self.connection()?, |connection| {
            register(connection, registration)
        })
    }

    fn connect_machine(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        hash: &TokenHash,
        contact: &Contact,
    ) -> Result<Option<Machine>> {
        transaction(&mut *self.connection()?, |connection| {
            connect(connection, workspace_id, machine_id, hash, contact)
        })
    }

    fn record_contact(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        contact: &Contact,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            write_contact(connection, workspace_id, machine_id, contact, false)
        })
    }

    fn record_activity(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        session_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_machine(connection, workspace_id, machine_id)?;
            if session(connection, workspace_id, machine_id)? != Some(session_id) {
                return Err(Error::Conflict);
            }
            diesel::update(
                machines::table
                    .filter(machines::workspace_id.eq(workspace_id))
                    .find(machine_id),
            )
            .set(machines::last_contact_at.eq(at))
            .execute(connection)
            .store()?;
            Ok(())
        })
    }

    fn machine_session(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<Option<Uuid>> {
        let mut connection = self.connection()?;
        let present = machines::table
            .filter(machines::workspace_id.eq(workspace_id))
            .filter(machines::removed_at.is_null())
            .find(machine_id)
            .select(machines::id)
            .first::<Uuid>(&mut *connection)
            .optional()
            .store()?;
        if present.is_none() {
            return Ok(None);
        }
        session(&mut connection, workspace_id, machine_id)
    }

    fn machine_snapshots(&self, workspace_id: Uuid) -> Result<Vec<MachineSnapshot>> {
        let machines = self.machines(workspace_id)?;
        let mut connection = self.connection()?;
        machines
            .into_iter()
            .map(|machine| {
                let raw = machine_reports::table
                    .filter(machine_reports::workspace_id.eq(workspace_id))
                    .filter(machine_reports::machine_id.eq(machine.id))
                    .select(machine_reports::report)
                    .first::<serde_json::Value>(&mut *connection)
                    .optional()
                    .store()?;
                let report = raw.map(serde_json::from_value).transpose().internal()?;
                Ok(MachineSnapshot { machine, report })
            })
            .collect()
    }

    fn set_machine_draining(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        draining: bool,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_machine(connection, workspace_id, machine_id)?;
            diesel::update(
                machines::table
                    .filter(machines::workspace_id.eq(workspace_id))
                    .find(machine_id),
            )
            .set(machines::draining.eq(draining))
            .execute(connection)
            .store()?;
            Ok(())
        })
    }

    fn revoke_machine(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            lock_workspace(connection, workspace_id)?;
            let count = remove_machine_records(connection, workspace_id, machine_id, at).store()?;
            require_record(count)
        })
    }
}
