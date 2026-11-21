//! Workspace-scoped durable payment recovery queries.

use ciproxy_schema::billing::billing_payments;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::billing::payments::PaymentWork;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};

pub(super) fn read(
    connection: &mut PgConnection,
    workspace: Uuid,
    id: Uuid,
) -> Result<Option<PaymentWork>> {
    let json = billing_payments::table
        .find(id)
        .filter(billing_payments::workspace_id.eq(workspace))
        .select(billing_payments::state_json)
        .first::<String>(connection)
        .optional()
        .store()?;
    json.map(|json| serde_json::from_str(&json).internal())
        .transpose()
}

pub(super) fn pending(connection: &mut PgConnection, workspace: Uuid) -> Result<Vec<PaymentWork>> {
    billing_payments::table
        .filter(billing_payments::workspace_id.eq(workspace))
        .filter(billing_payments::pending.eq(true))
        .select(billing_payments::state_json)
        .load::<String>(connection)
        .store()?
        .into_iter()
        .map(|json| serde_json::from_str(&json).internal())
        .collect()
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace: Uuid,
    work: &PaymentWork,
) -> Result<()> {
    if work.workspace_id != workspace {
        return Err(Error::InvalidReference);
    }
    let json = serde_json::to_string(work).internal()?;
    let changed = connection
        .transaction::<_, diesel::result::Error, _>(|connection| {
            workspaces::table
                .find(workspace)
                .select(workspaces::id)
                .for_update()
                .first::<Uuid>(connection)?;
            let changed = diesel::update(
                billing_payments::table
                    .find(work.id)
                    .filter(billing_payments::workspace_id.eq(workspace)),
            )
            .set((
                billing_payments::state_json.eq(&json),
                billing_payments::pending.eq(work.pending),
            ))
            .execute(connection)?;
            if changed != 0 {
                return Ok(changed);
            }
            diesel::insert_into(billing_payments::table)
                .values((
                    billing_payments::command_id.eq(work.id),
                    billing_payments::workspace_id.eq(workspace),
                    billing_payments::state_json.eq(&json),
                    billing_payments::pending.eq(work.pending),
                ))
                .on_conflict_do_nothing()
                .execute(connection)
        })
        .store()?;
    require_record(changed)
}
