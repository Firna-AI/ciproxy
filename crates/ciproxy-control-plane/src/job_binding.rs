//! Save authenticated Worker identity before routing a preparation decision.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_store_interface::job_bindings::{OwnJobBinding, WorkerJobId};
use ciproxy_store_interface::own_runners::OwnRunner;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn bind_worker_job(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        runner: &OwnRunner,
        job_id: JobId,
    ) -> Result<()> {
        let Some(github_runner_id) = runner.runner.github_id else {
            tracing::warn!(runner=%runner.runner.id, "job binding waits for the saved runner registration");
            return Ok(());
        };
        let binding = OwnJobBinding {
            runner_id: runner.runner.id,
            organization_id: runner.runner.organization_id,
            machine_id,
            github_runner_id,
            worker_job_id: WorkerJobId(job_id),
        };
        let store = self.store.clone();
        let applied =
            tokio::task::spawn_blocking(move || store.bind_own_job(workspace_id, &binding))
                .await
                .internal()?
                .internal()?;
        if !applied {
            tracing::warn!(runner=%runner.runner.id, "job binding kept the saved identity or changed runner authority");
        }
        Ok(())
    }
}
