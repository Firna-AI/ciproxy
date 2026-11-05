//! Scoped renewal outcome queries through an exclusive billing session.

use ciproxy_schema::billing::billing_renewals;
use ciproxy_store_interface::billing::renewals::Renewal;
use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

pub(super) fn read(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    subscription_id: &str,
) -> Result<Vec<Renewal>> {
    let rows = billing_renewals::table
        .filter(billing_renewals::workspace_id.eq(workspace_id))
        .filter(billing_renewals::subscription_id.eq(subscription_id))
        .order(billing_renewals::invoice_id)
        .select(billing_renewals::outcome_json)
        .load::<String>(connection)
        .store()?;
    rows.into_iter()
        .map(|json| serde_json::from_str(&json).internal())
        .collect()
}

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    renewal: &Renewal,
) -> Result<()> {
    let json = serde_json::to_string(renewal).internal()?;
    diesel::insert_into(billing_renewals::table)
        .values((
            billing_renewals::workspace_id.eq(workspace_id),
            billing_renewals::subscription_id.eq(&renewal.subscription_id),
            billing_renewals::invoice_id.eq(&renewal.invoice_id),
            billing_renewals::outcome_json.eq(&json),
        ))
        .on_conflict((billing_renewals::workspace_id, billing_renewals::invoice_id))
        .do_update()
        .set((
            billing_renewals::subscription_id.eq(&renewal.subscription_id),
            billing_renewals::outcome_json.eq(&json),
        ))
        .execute(connection)
        .store()?;
    Ok(())
}
