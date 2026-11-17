//! Short leased result sections shared by independently owned remote tasks.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runtime_results::{OwnRunnerGuard, OwnRunnerResult};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn boat_operation(&self, workspace: Uuid) -> Arc<Mutex<()>> {
        self.boat_operations
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
        if guard.runner != *runner {
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
