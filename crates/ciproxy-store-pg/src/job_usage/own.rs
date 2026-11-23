//! Exact own-machine usage limits without a workspace routing snapshot.

use chrono::{DateTime, Utc};
use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::job_usage::OwnUsageTarget;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;

/// Read limits only for a runner of the specified machine and workspace.
pub(super) fn target(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    machine_id: Uuid,
    runner_id: Uuid,
) -> Result<Option<OwnUsageTarget>> {
    let target = runners::table
        .inner_join(
            own_runner_commands::table.on(own_runner_commands::runner_id
                .eq(runners::id)
                .and(own_runner_commands::workspace_id.eq(runners::workspace_id))),
        )
        .filter(runners::id.eq(runner_id))
        .filter(runners::workspace_id.eq(workspace_id))
        .filter(runners::machine_id.eq(machine_id))
        .select((
            runners::vcpus,
            runners::memory_bytes,
            own_runner_commands::issued_at,
            own_runner_commands::confirmed_at,
        ))
        .first::<(i32, i64, DateTime<Utc>, Option<DateTime<Utc>>)>(connection)
        .optional()
        .store()?;
    target
        .map(|(vcpus, memory_bytes, issued_at, confirmed_at)| {
            Ok(OwnUsageTarget {
                vcpus: u32::try_from(vcpus).internal()?,
                memory_bytes: u64::try_from(memory_bytes).internal()?,
                started_at: issued_at.min(confirmed_at.unwrap_or(issued_at)),
            })
        })
        .transpose()
}
