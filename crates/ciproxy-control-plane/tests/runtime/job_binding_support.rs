//! Arrange an already verified handoff for tests of later runtime work.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_store_interface::job_bindings::{OwnJobBinding, WorkerJobId};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

pub(super) fn bind_job(
    store: &dyn Store,
    workspace: Uuid,
    runner: &OwnRunner,
    worker_job_id: JobId,
) {
    let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
        panic!("own-machine handoff requires a machine runner")
    };
    assert!(
        store
            .bind_own_job(
                workspace,
                &OwnJobBinding {
                    runner_id: runner.runner.id,
                    organization_id: runner.runner.organization_id,
                    machine_id,
                    github_runner_id: runner.runner.github_id.unwrap(),
                    worker_job_id: WorkerJobId(worker_job_id),
                }
            )
            .unwrap()
    );
}
