//! Ciphertext-only Boat key rotation, retention, and erasure queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::boat::boat_accounts;
use ciproxy_schema::settings::settings;
use ciproxy_store_interface::boat::{BoatAccount, BoatStore};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(Queryable, Selectable)]
#[diesel(table_name = boat_accounts)]
struct AccountRow {
    id: Uuid,
    account_name: String,
    connected_at: DateTime<Utc>,
    retired_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
}

impl From<AccountRow> for BoatAccount {
    fn from(row: AccountRow) -> Self {
        Self {
            id: row.id,
            account_name: row.account_name,
            connected_at: row.connected_at,
            retired_at: row.retired_at,
            rejected_at: row.rejected_at,
        }
    }
}

impl BoatStore for PgStore {
    fn connect_boat(
        &self,
        organization_id: Uuid,
        account: &BoatAccount,
        key: &EncryptedBoatKey,
        spend_limit_usd_micros: i64,
    ) -> Result<()> {
        self.connection()?
            .transaction(|connection| {
                let count = diesel::update(settings::table.find(organization_id))
                    .set(settings::boat_spend_limit_usd_micros.eq(spend_limit_usd_micros))
                    .execute(connection)?;
                if count == 0 {
                    return Err(diesel::result::Error::NotFound);
                }
                diesel::update(
                    boat_accounts::table
                        .filter(boat_accounts::organization_id.eq(organization_id))
                        .filter(boat_accounts::retired_at.is_null()),
                )
                .set(boat_accounts::retired_at.eq(account.connected_at))
                .execute(connection)?;
                diesel::insert_into(boat_accounts::table)
                    .values((
                        boat_accounts::id.eq(account.id),
                        boat_accounts::organization_id.eq(organization_id),
                        boat_accounts::account_name.eq(&account.account_name),
                        boat_accounts::key_ciphertext.eq(key.as_bytes()),
                        boat_accounts::connected_at.eq(account.connected_at),
                    ))
                    .execute(connection)?;
                QueryResult::Ok(())
            })
            .store()
    }

    fn boat_accounts(&self, organization_id: Uuid) -> Result<Vec<BoatAccount>> {
        Ok(boat_accounts::table
            .filter(boat_accounts::organization_id.eq(organization_id))
            .order(boat_accounts::connected_at)
            .select(AccountRow::as_select())
            .load::<AccountRow>(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(BoatAccount::from)
            .collect())
    }

    fn encrypted_boat_key(
        &self,
        organization_id: Uuid,
        id: Uuid,
    ) -> Result<Option<EncryptedBoatKey>> {
        Ok(boat_accounts::table
            .filter(boat_accounts::organization_id.eq(organization_id))
            .find(id)
            .select(boat_accounts::key_ciphertext)
            .first::<Vec<u8>>(&mut *self.connection()?)
            .optional()
            .store()?
            .map(EncryptedBoatKey::new))
    }

    fn reject_boat_key(&self, organization_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        let count = diesel::update(
            boat_accounts::table
                .filter(boat_accounts::organization_id.eq(organization_id))
                .find(id),
        )
        .set(boat_accounts::rejected_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }

    fn disconnect_boat(&self, organization_id: Uuid, at: DateTime<Utc>) -> Result<()> {
        diesel::update(
            boat_accounts::table
                .filter(boat_accounts::organization_id.eq(organization_id))
                .filter(boat_accounts::retired_at.is_null()),
        )
        .set(boat_accounts::retired_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        Ok(())
    }

    fn delete_unused_boat_keys(&self, organization_id: Uuid) -> Result<usize> {
        diesel::sql_query(
            "DELETE FROM boat_accounts AS account
             WHERE account.organization_id = $1 AND account.retired_at IS NOT NULL
             AND NOT EXISTS (
                 SELECT 1 FROM runners
                 WHERE runners.organization_id = account.organization_id
                 AND runners.boat_key_id = account.id AND runners.state <> 'stopped'
             )",
        )
        .bind::<sql_types::Uuid, _>(organization_id)
        .execute(&mut *self.connection()?)
        .store()
    }
}
