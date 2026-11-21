//! Private row mappings and coherent workspace overflow snapshots.

use chrono::{DateTime, Utc};
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_schema::settings::settings;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use diesel::prelude::*;
use diesel::sql_types::{Binary, Nullable, Text, Timestamptz};
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::read_snapshot;

#[derive(QueryableByName)]
/// Private stored fields shared by account snapshots and startup authority reads.
pub(super) struct AccountRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    account_name: String,
    #[diesel(sql_type = Binary)]
    key_ciphertext: Vec<u8>,
    #[diesel(sql_type = Timestamptz)]
    connected_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    retired_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    rejected_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    state_json: Option<String>,
}

impl AccountRow {
    pub(super) fn into_record(self) -> Result<AccountRecord> {
        Ok(AccountRecord {
            account: BoatAccount {
                id: self.id,
                account_name: self.account_name,
                connected_at: self.connected_at,
                retired_at: self.retired_at,
                rejected_at: self.rejected_at,
            },
            key: EncryptedBoatKey::new(self.key_ciphertext),
            state: match self.state_json {
                Some(json) => serde_json::from_str(&json).internal()?,
                None => AccountState {
                    wallet: "personal".to_owned(),
                    ..AccountState::default()
                },
            },
        })
    }
}

pub(crate) fn snapshot(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<OverflowSnapshot> {
    read_snapshot(connection, |connection| load(connection, workspace_id))
}

/// Read account and delivery rows inside the caller's existing transaction.
pub(crate) fn load(connection: &mut PgConnection, workspace_id: Uuid) -> Result<OverflowSnapshot> {
    let (enabled, spend_limit_usd_micros) = settings::table
        .find(workspace_id)
        .select((
            settings::overflow_enabled,
            settings::boat_spend_limit_usd_micros,
        ))
        .first::<(bool, Option<i64>)>(connection)
        .store()?;
    let accounts = diesel::sql_query(
        "SELECT a.id, a.account_name, a.key_ciphertext, a.connected_at,
                    a.retired_at, a.rejected_at, s.state_json
                 FROM boat_accounts a LEFT JOIN overflow_accounts s ON s.key_id = a.id
                 WHERE a.workspace_id = $1 ORDER BY a.connected_at, a.id",
    )
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .load::<AccountRow>(connection)
    .store()?
    .into_iter()
    .map(AccountRow::into_record)
    .collect::<Result<Vec<_>>>()?;
    let starts = overflow_starts::table
        .filter(overflow_starts::workspace_id.eq(workspace_id))
        .select(overflow_starts::record_json)
        .order(overflow_starts::delivery_id)
        .load::<String>(connection)
        .store()?
        .into_iter()
        .map(|json| serde_json::from_str(&json).internal())
        .collect::<Result<Vec<_>>>()?;
    Ok(OverflowSnapshot {
        accounts,
        starts,
        enabled,
        spend_limit_usd_micros,
    })
}
