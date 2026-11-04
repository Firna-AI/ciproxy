//! Atomic installation and authenticated connection queries.

use ciproxy_schema::machines::machines;
use ciproxy_schema::tokens::tokens;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::gateway::{Contact, Registration};
use ciproxy_store_interface::machines::Machine;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_contacts::{lock_machine, write_contact};
use crate::machine_observation::MachineObservation;
use crate::machine_reads::load_machine;
use crate::tokens::insert_token;

pub(crate) fn lock_workspace(connection: &mut PgConnection, workspace_id: Uuid) -> Result<()> {
    workspaces::table
        .find(workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    Ok(())
}

pub(crate) fn register(connection: &mut PgConnection, registration: &Registration) -> Result<bool> {
    let workspace = registration.workspace_id;
    match lock_workspace(connection, workspace) {
        Ok(()) => (),
        Err(Error::NotFound) => return Ok(false),
        Err(error) => return Err(error),
    }
    let deleting = workspaces::table
        .find(workspace)
        .select(workspaces::delete_after)
        .first::<Option<chrono::DateTime<chrono::Utc>>>(connection)
        .store()?;
    if deleting.is_some() {
        return Ok(false);
    }
    let token = tokens::table
        .filter(tokens::workspace_id.eq(workspace))
        .filter(tokens::token_hash.eq(registration.install_hash.as_bytes()))
        .filter(tokens::purpose.eq("install"))
        .filter(tokens::revoked_at.is_null())
        .filter(tokens::expires_at.gt(registration.contact.at))
        .select(tokens::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()
        .store()?;
    if token.is_none() {
        return Ok(false);
    }
    let observed = MachineObservation::from_contact(&registration.contact);
    diesel::insert_into(machines::table)
        .values((
            machines::id.eq(registration.machine_id),
            machines::workspace_id.eq(workspace),
            &observed,
        ))
        .execute(connection)
        .store()?;
    insert_token(
        connection,
        &Token {
            id: registration.credential_id,
            workspace_id: workspace,
            subject: TokenSubject::Machine(registration.machine_id),
            created_at: registration.contact.at,
            expires_at: None,
            revoked_at: None,
        },
        &registration.credential_hash,
    )
    .store()?;
    write_contact(
        connection,
        workspace,
        registration.machine_id,
        &registration.contact,
        true,
    )?;
    Ok(true)
}

pub(crate) fn connect(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
    hash: &TokenHash,
    contact: &Contact,
) -> Result<Option<Machine>> {
    match lock_workspace(connection, workspace_id) {
        Ok(()) => (),
        Err(Error::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    }
    match lock_machine(connection, workspace_id, machine_id) {
        Ok(()) => (),
        Err(Error::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    }
    let token = tokens::table
        .filter(tokens::workspace_id.eq(workspace_id))
        .filter(tokens::machine_id.eq(machine_id))
        .filter(tokens::purpose.eq("machine"))
        .filter(tokens::token_hash.eq(hash.as_bytes()))
        .filter(tokens::revoked_at.is_null())
        .select(tokens::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()
        .store()?;
    if token.is_none() {
        return Ok(None);
    }
    write_contact(connection, workspace_id, machine_id, contact, true)?;
    Ok(Some(load_machine(connection, workspace_id, machine_id)?))
}
