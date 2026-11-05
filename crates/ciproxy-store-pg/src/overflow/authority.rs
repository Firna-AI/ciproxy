//! Shared current authority for a proposed or durably saved delivery.

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::store::AccountRecord;
use diesel::prelude::*;
use diesel::sql_types::Text;
use uuid::Uuid;

use crate::error::QueryResultExt;

use super::rows::AccountRow;

const ACCOUNT: &str = "
    SELECT a.id, a.account_name, a.key_ciphertext, a.connected_at,
           a.retired_at, a.rejected_at, s.state_json
    FROM delivery d
    JOIN boat_accounts a ON a.id = d.key_id AND a.workspace_id = d.workspace_id
    JOIN organizations o ON o.id = (d.record_json::jsonb ->> 'organization_id')::uuid
        AND o.workspace_id = d.workspace_id
    JOIN settings w ON w.workspace_id = d.workspace_id
    JOIN scale_sets ss ON ss.id = (d.record_json::jsonb ->> 'scale_set_id')::uuid
        AND ss.workspace_id = d.workspace_id AND ss.organization_id = o.id
    JOIN labels l ON l.id = (d.record_json::jsonb ->> 'label_id')::uuid
        AND l.workspace_id = d.workspace_id AND ss.label_id = l.id
    LEFT JOIN overflow_accounts s ON s.key_id = a.id
    WHERE a.id = (d.record_json::jsonb ->> 'key_id')::uuid
        AND o.app_state = 'active' AND w.overflow_enabled
        AND a.retired_at IS NULL AND a.rejected_at IS NULL
        AND l.allow_boat
        AND l.vcpus = (d.record_json::jsonb ->> 'requested_vcpus')::bigint
        AND l.memory_gb = (d.record_json::jsonb ->> 'requested_memory_gb')::bigint
        AND l.disk_gb = (d.record_json::jsonb ->> 'requested_disk_gb')::bigint";

pub(super) fn startup_account(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<AccountRecord>> {
    diesel::sql_query(format!(
        "WITH delivery AS (SELECT workspace_id, key_id, record_json FROM overflow_starts
         WHERE workspace_id = $1 AND delivery_id = $2) {ACCOUNT}"
    ))
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Uuid, _>(delivery_id)
    .get_result::<AccountRow>(connection)
    .optional()
    .store()?
    .map(AccountRow::into_record)
    .transpose()
}

pub(super) fn permits_request(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    key_id: Uuid,
    json: &str,
) -> Result<bool> {
    Ok(diesel::sql_query(format!(
        "WITH delivery AS (SELECT $1::uuid AS workspace_id, $2::uuid AS key_id,
         $3::text AS record_json) {ACCOUNT}"
    ))
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Uuid, _>(key_id)
    .bind::<Text, _>(json)
    .get_result::<AccountRow>(connection)
    .optional()
    .store()?
    .is_some())
}
