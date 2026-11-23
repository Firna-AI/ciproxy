//! Short leased result sections shared by independently owned remote tasks.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::runtime_results::{OwnRunnerGuard, OwnRunnerResult};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn overflow_operation(&self, workspace: Uuid) -> Arc<Mutex<()>> {
        self.overflow_operations
            .lock()
            .await
            .entry(workspace)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub(crate) fn dispatch(
        &self,
        key: Work,
        future: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Result<()> {
        self.task_scope()?.dispatch(key, Box::pin(future))
    }

    pub(crate) async fn runner_guard(
        &self,
        workspace: Uuid,
        runner: &OwnRunner,
    ) -> Result<OwnRunnerGuard> {
        let store = self.store.clone();
        let id = runner.runner.id;
        let guard = tokio::task::spawn_blocking(move || store.own_runner_guard(workspace, id))
            .await
            .internal()?
            .internal()?
            .ok_or(Error::StateChanged)?;
        if dispatch_authority(&guard.runner) != dispatch_authority(runner) {
            return Err(Error::StateChanged);
        }
        Ok(guard)
    }

    pub(crate) async fn apply_remote(
        &self,
        workspace: Uuid,
        expected: OwnRunnerGuard,
        result: OwnRunnerResult,
    ) -> Result<bool> {
        let _operation = self.operation.lock().await;
        let Some(lease) = self.store.own_runner_lease(workspace).await.internal()? else {
            return Ok(false);
        };
        lease.check().await.internal()?;
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            store.apply_own_runner_result(workspace, &expected, result)
        })
        .await
        .internal()?
        .internal()
    }
}

/// Dispatch ignores the caller's own evidence writes and uses the saved full guard.
#[derive(PartialEq)]
struct DispatchAuthority<'a> {
    runner_id: Uuid,
    organization_id: Uuid,
    label_id: Uuid,
    scale_set_id: Uuid,
    github_id: Option<i64>,
    state: RunnerState,
    place: &'a RunnerPlace,
    registration_scale_set_id: Option<i64>,
    label_name: &'a str,
    command_id: Uuid,
}

fn dispatch_authority(runner: &OwnRunner) -> DispatchAuthority<'_> {
    let OwnRunner {
        worker_job_id: _,
        registration_scale_set_id,
        requested_size: _,
        slot_indices: _,
        runner,
        label_name,
        command_id,
        issued_at: _,
        acknowledged: _,
        confirmed_at: _,
    } = runner;
    let Runner {
        id,
        organization_id,
        label_id,
        scale_set_id,
        github_id,
        state,
        size: _,
        place,
    } = runner;
    DispatchAuthority {
        runner_id: *id,
        organization_id: *organization_id,
        label_id: *label_id,
        scale_set_id: *scale_set_id,
        github_id: *github_id,
        state: *state,
        place,
        registration_scale_set_id: *registration_scale_set_id,
        label_name,
        command_id: *command_id,
    }
}
