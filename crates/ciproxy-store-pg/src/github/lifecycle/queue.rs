//! Immutable event insertion, ordered reads, and delete-after-accept confirmation.

use ciproxy_schema::github::github_lifecycle_outbox;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github_lifecycle::{LifecycleDelivery, LifecycleEvent};
use diesel::prelude::*;
use diesel::result::Error;
use serde_json::Value;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

pub(super) fn insert(connection: &mut PgConnection, event: &LifecycleEvent) -> QueryResult<()> {
    let body = match serde_json::to_value(event) {
        Ok(body) => body,
        Err(source) => return Err(Error::SerializationError(Box::new(source))),
    };
    diesel::insert_into(github_lifecycle_outbox::table)
        .values((
            github_lifecycle_outbox::github_id.eq(event.organization().github_id),
            github_lifecycle_outbox::event.eq(body),
        ))
        .execute(connection)?;
    Ok(())
}

pub(super) fn organizations(store: &PgStore) -> Result<Vec<i64>> {
    github_lifecycle_outbox::table
        .select(github_lifecycle_outbox::github_id)
        .distinct()
        .order(github_lifecycle_outbox::github_id)
        .load(&mut *store.connection()?)
        .store()
}

pub(super) fn pending(store: &PgStore, github_id: i64) -> Result<Vec<LifecycleDelivery>> {
    github_lifecycle_outbox::table
        .filter(github_lifecycle_outbox::github_id.eq(github_id))
        .order(github_lifecycle_outbox::sequence)
        .select((
            github_lifecycle_outbox::sequence,
            github_lifecycle_outbox::event,
        ))
        .load::<(i64, Value)>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|(sequence, body)| {
            Ok(LifecycleDelivery {
                sequence,
                event: serde_json::from_value(body).internal()?,
            })
        })
        .collect()
}

pub(super) fn confirm(store: &PgStore, github_id: i64, sequence: i64) -> Result<()> {
    let count = diesel::delete(
        github_lifecycle_outbox::table
            .filter(github_lifecycle_outbox::github_id.eq(github_id))
            .filter(github_lifecycle_outbox::sequence.eq(sequence)),
    )
    .execute(&mut *store.connection()?)
    .store()?;
    require_record(count)
}
