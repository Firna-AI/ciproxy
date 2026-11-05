//! Pending connection grants issued before provider or encryption work.

use ciproxy_schema::organizations::organizations;
use ciproxy_schema::overflow::overflow_connection_grants;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::connection::ConnectionGrant;
use diesel::dsl::exists;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

pub(super) fn begin(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    key_id: Uuid,
) -> Result<Option<ConnectionGrant>> {
    transaction(connection, |connection| {
        let workspace = workspaces::table
            .find(workspace_id)
            .select(workspaces::id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()
            .store()?;
        if workspace.is_none() {
            return Ok(None);
        }
        let permitted = diesel::select(exists(
            organizations::table
                .filter(organizations::workspace_id.eq(workspace_id))
                .filter(organizations::app_state.ne("removed")),
        ))
        .get_result::<bool>(connection)
        .store()?;
        if !permitted {
            return Ok(None);
        }
        diesel::delete(
            overflow_connection_grants::table
                .filter(overflow_connection_grants::workspace_id.eq(workspace_id)),
        )
        .execute(connection)
        .store()?;
        let id = diesel::insert_into(overflow_connection_grants::table)
            .values((
                overflow_connection_grants::workspace_id.eq(workspace_id),
                overflow_connection_grants::key_id.eq(key_id),
            ))
            .returning(overflow_connection_grants::id)
            .get_result::<Uuid>(connection)
            .store()?;
        Ok(Some(ConnectionGrant { id }))
    })
}
