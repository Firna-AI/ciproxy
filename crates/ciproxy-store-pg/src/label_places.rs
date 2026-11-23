//! Queries for the ordered places of a workflow label.

use std::collections::HashMap;

use ciproxy_schema::labels::{label_places, labels};
use ciproxy_store_interface::labels::LabelPlace;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use diesel::FromSqlRow;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::sql_types::{self, Text};
use uuid::Uuid;

use crate::overflow_provider::DbOverflowProvider;

#[derive(FromSqlRow)]
#[diesel(sql_type = Text)]
struct DbLabelPlace(LabelPlace);

impl FromSql<Text, Pg> for DbLabelPlace {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        if value.as_bytes() == b"machine" {
            return Ok(Self(LabelPlace::Machine));
        }
        Ok(Self(LabelPlace::Provider(
            DbOverflowProvider::from_sql(value)?.0,
        )))
    }
}

/// Read a scoped label's places in position order.
pub(crate) fn load(connection: &mut PgConnection, label_id: Uuid) -> QueryResult<Vec<LabelPlace>> {
    Ok(label_places::table
        .filter(label_places::label_id.eq(label_id))
        .order(label_places::position)
        .select(label_places::place)
        .load::<DbLabelPlace>(connection)?
        .into_iter()
        .map(|place| place.0)
        .collect())
}

/// Read the places of all active labels in a workspace with one query.
pub(crate) fn load_workspace(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<HashMap<Uuid, Vec<LabelPlace>>> {
    let rows = label_places::table
        .inner_join(labels::table)
        .filter(labels::workspace_id.eq(workspace_id))
        .filter(labels::deleted_at.is_null())
        .order((label_places::label_id, label_places::position))
        .select((label_places::label_id, label_places::place))
        .load::<(Uuid, DbLabelPlace)>(connection)?;
    let mut orders: HashMap<_, Vec<_>> = HashMap::new();
    for (label, place) in rows {
        orders.entry(label).or_default().push(place.0);
    }
    Ok(orders)
}

/// Save the initial entries of an order or the entries of a cleared order.
pub(crate) fn insert(
    connection: &mut PgConnection,
    label_id: Uuid,
    order: &[LabelPlace],
) -> QueryResult<()> {
    let rows: Vec<_> = (0..)
        .zip(order)
        .map(|(position, place)| {
            (
                label_places::label_id.eq(label_id),
                label_places::position.eq(position),
                label_places::place.eq(place.as_str()),
            )
        })
        .collect();
    diesel::insert_into(label_places::table)
        .values(rows)
        .execute(connection)?;
    Ok(())
}

/// Clear the order of a scoped label within its write transaction.
pub(crate) fn clear(connection: &mut PgConnection, label_id: Uuid) -> QueryResult<()> {
    diesel::delete(label_places::table.filter(label_places::label_id.eq(label_id)))
        .execute(connection)?;
    Ok(())
}

/// Append a first offer to active Linux labels that do not list the provider.
pub(crate) fn append_provider(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    provider: OverflowProvider,
) -> QueryResult<()> {
    diesel::sql_query(
        "INSERT INTO label_places (label_id, position, place)
         SELECT l.id, COALESCE(MAX(p.position) + 1, 0), $2 FROM labels l
         LEFT JOIN label_places p ON p.label_id = l.id
         WHERE l.workspace_id = $1 AND l.platform = 'linux_x64' AND l.deleted_at IS NULL
         GROUP BY l.id
         ON CONFLICT (label_id, place) DO NOTHING",
    )
    .bind::<sql_types::Uuid, _>(workspace_id)
    .bind::<Text, _>(DbOverflowProvider(provider))
    .execute(connection)?;
    Ok(())
}
