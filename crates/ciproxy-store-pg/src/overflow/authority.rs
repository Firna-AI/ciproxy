//! One current authority read bound to a workspace's durable delivery identity.

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::store::AccountRecord;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

use super::rows::AccountRow;

pub(super) fn startup_account(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<AccountRecord>> {
    diesel::sql_query(
        "SELECT a.id, a.account_name, a.key_ciphertext, a.connected_at,
                a.retired_at, a.rejected_at, s.state_json
         FROM overflow_starts d
         JOIN boat_accounts a ON a.id = d.key_id AND a.workspace_id = d.workspace_id
         JOIN organizations o ON o.id = (d.record_json::jsonb ->> 'organization_id')::uuid
             AND o.workspace_id = d.workspace_id
         JOIN settings w ON w.workspace_id = d.workspace_id
         LEFT JOIN overflow_accounts s ON s.key_id = a.id
         WHERE d.workspace_id = $1 AND d.delivery_id = $2
             AND a.id = (d.record_json::jsonb ->> 'key_id')::uuid
             AND o.app_state = 'active' AND w.overflow_enabled
             AND a.retired_at IS NULL AND a.rejected_at IS NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Uuid, _>(delivery_id)
    .get_result::<AccountRow>(connection)
    .optional()
    .store()?
    .map(AccountRow::into_record)
    .transpose()
}
