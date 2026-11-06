//! Scoped runner state and first execution evidence commit in one transaction.

use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::own_runners::OwnRunnerObservation;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Timestamptz};
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;

pub(crate) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    organization_id: Uuid,
    runner_id: Uuid,
    observation: OwnRunnerObservation,
) -> Result<()> {
    transaction(connection, |connection| {
        let state: &str = observation.state.into();
        let count = diesel::update(
            runners::table
                .filter(runners::workspace_id.eq(workspace_id))
                .filter(runners::organization_id.eq(organization_id))
                .find(runner_id),
        )
        .set((
            runners::state.eq(state),
            runners::github_id.eq(observation.github_id),
        ))
        .execute(connection)
        .store()?;
        require_record(count)?;
        let count = diesel::update(
            own_runner_commands::table
                .filter(own_runner_commands::workspace_id.eq(workspace_id))
                .find(runner_id),
        )
        .set(
            own_runner_commands::confirmed_at.eq(diesel::dsl::sql::<Nullable<Timestamptz>>(
                "COALESCE(confirmed_at, ",
            )
            .bind::<Timestamptz, _>(observation.observed_at)
            .sql(")")),
        )
        .execute(connection)
        .store()?;
        require_record(count)
    })
}
