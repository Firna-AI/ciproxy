//! Scoped paid quotes and atomic quote consumption with durable commands.

use ciproxy_schema::billing::{billing_accounts, billing_quotes};
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::quotes::StoredQuote;
use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

pub(super) fn read(
    connection: &mut PgConnection,
    workspace: Uuid,
    id: Uuid,
) -> Result<Option<StoredQuote>> {
    let row = billing_quotes::table
        .find(id)
        .filter(billing_quotes::workspace_id.eq(workspace))
        .select((billing_quotes::quote_json, billing_quotes::command_id))
        .first::<(String, Option<Uuid>)>(connection)
        .optional()
        .store()?;
    let Some((json, command_id)) = row else {
        return Ok(None);
    };
    let mut quote: StoredQuote = serde_json::from_str(&json).internal()?;
    quote.command_id = command_id;
    Ok(Some(quote))
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace: Uuid,
    quote: &StoredQuote,
) -> Result<()> {
    let json = serde_json::to_string(quote).internal()?;
    connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            workspaces::table
                .find(workspace)
                .select(workspaces::id)
                .for_update()
                .first::<Uuid>(connection)?;
            let pending = billing_accounts::table
                .find(workspace)
                .select(billing_accounts::command_json)
                .first::<Option<String>>(connection)?;
            if pending.is_none() {
                diesel::delete(
                    billing_quotes::table
                        .filter(billing_quotes::workspace_id.eq(workspace))
                        .filter(billing_quotes::expires_at.le(quote.payment.proration_at)),
                )
                .execute(connection)?;
            }
            diesel::insert_into(billing_quotes::table)
                .values((
                    billing_quotes::id.eq(quote.payment.id),
                    billing_quotes::workspace_id.eq(workspace),
                    billing_quotes::quote_json.eq(&json),
                    billing_quotes::expires_at.eq(quote.expires_at),
                    billing_quotes::command_id.eq(quote.command_id),
                ))
                .execute(connection)?;
            Ok(())
        })
        .store()
}

pub(super) fn consume(
    connection: &mut PgConnection,
    workspace: Uuid,
    quote_id: Uuid,
    command: &BillingCommand,
) -> Result<bool> {
    let json = serde_json::to_string(command).internal()?;
    connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            workspaces::table
                .find(workspace)
                .select(workspaces::id)
                .for_update()
                .first::<Uuid>(connection)?;
            if billing_accounts::table
                .find(workspace)
                .select(billing_accounts::command_json)
                .first::<Option<String>>(connection)?
                .is_some()
            {
                return Ok(false);
            }
            let changed = diesel::update(
                billing_quotes::table
                    .find(quote_id)
                    .filter(billing_quotes::workspace_id.eq(workspace))
                    .filter(billing_quotes::command_id.is_null()),
            )
            .set(billing_quotes::command_id.eq(command.id))
            .execute(connection)?;
            if changed == 0 {
                return Ok(false);
            }
            diesel::update(billing_accounts::table.find(workspace))
                .set(billing_accounts::command_json.eq(&json))
                .execute(connection)?;
            Ok(true)
        })
        .store()
}
