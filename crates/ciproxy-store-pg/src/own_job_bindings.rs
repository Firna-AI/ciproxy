//! Atomic immutable Worker job bindings for exact registered machine runners.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::job_bindings::{OwnJobBinding, WorkerJobId};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

pub(crate) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    binding: &OwnJobBinding,
) -> Result<bool> {
    transaction(connection, |connection| {
        let runner = runners::table
            .find(binding.runner_id)
            .filter(runners::workspace_id.eq(workspace_id))
            .filter(runners::organization_id.eq(binding.organization_id))
            .filter(runners::machine_id.eq(binding.machine_id))
            .filter(runners::github_id.eq(binding.github_runner_id))
            .filter(runners::state.ne("stopped"))
            .select(runners::id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()
            .store()?;
        if runner.is_none() {
            return Ok(false);
        }
        let WorkerJobId(JobId(worker_job_id)) = &binding.worker_job_id;
        let changed = diesel::update(
            own_runner_commands::table
                .find(binding.runner_id)
                .filter(own_runner_commands::workspace_id.eq(workspace_id))
                .filter(
                    own_runner_commands::worker_job_id
                        .is_null()
                        .or(own_runner_commands::worker_job_id.eq(worker_job_id)),
                ),
        )
        .set(own_runner_commands::worker_job_id.eq(worker_job_id))
        .execute(connection)
        .store()?;
        Ok(changed == 1)
    })
}
