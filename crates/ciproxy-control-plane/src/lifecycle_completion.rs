//! Shared saved-state acceptance for take-out callbacks and explicit completion.

use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::reservations::retirement_results;
use crate::retirement::RunnerRetirement;
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    /// Join lifecycle work and check the production acceptance predicate.
    pub async fn quiesce_lifecycle(&self, event: &LifecycleEvent) -> Result<()> {
        self.task_scope()?.lifecycle_quiesce().await?;
        let LifecycleEvent::Removed {
            organization,
            cause: RemovalCause::WorkspaceTakeout,
            ..
        } = event
        else {
            return Ok(());
        };
        let _operation = self.operation.lock().await;
        let Some(lease) = self
            .store
            .own_runner_lease(organization.workspace_id)
            .await
            .internal()?
        else {
            return Err(Error::StateChanged);
        };
        lease.check().await.internal()?;
        self.takeout_accepted(organization).await
    }

    pub(crate) async fn current_overflow_busy(
        &self,
        workspace: Uuid,
        runner: Uuid,
    ) -> Result<bool> {
        let expected = self.overflow_answers.lock().await.get(&runner).cloned();
        let Some((generation, expected)) = expected else {
            return Ok(false);
        };
        let current_generation = self
            .takeout_checks
            .lock()
            .await
            .get(&expected.intent.organization_id)
            .filter(|check| !check.failed)
            .map(|check| check.generation);
        if current_generation != Some(generation) {
            return Ok(false);
        }
        if self.task_scope()?.contains(Work::OverflowRetire(runner)) {
            return Ok(false);
        }
        let store = self.store.clone();
        let current =
            tokio::task::spawn_blocking(move || store.overflow_runner_guard(workspace, runner))
                .await
                .internal()?
                .internal()?;
        Ok(current.as_ref() == Some(&expected))
    }

    pub(crate) async fn takeout_accepted(&self, organization: &Organization) -> Result<()> {
        let workspace = organization.workspace_id;
        let input = self.load(workspace).await?;
        let Some(current) = input
            .organizations
            .iter()
            .find(|current| current.id == organization.id)
        else {
            return Ok(());
        };
        if current.installation_id != organization.installation_id
            || current.github_id != organization.github_id
            || current.app_state != AppState::Removed
        {
            return Ok(());
        }
        let owner_removed = self
            .takeout_checks
            .lock()
            .await
            .get(&organization.id)
            .filter(|check| {
                &check.organization == current && (check.owner_removed || !check.failed)
            })
            .map(|check| check.owner_removed)
            .ok_or(Error::StateChanged)?;
        if owner_removed {
            return if input
                .runners
                .iter()
                .filter(|runner| runner.runner.organization_id == organization.id)
                .all(|runner| {
                    matches!(
                        runner.runner.state,
                        RunnerState::Stopping | RunnerState::Stopped
                    )
                }) {
                Ok(())
            } else {
                Err(Error::StateChanged)
            };
        }
        let store = self.store.clone();
        let organization_id = organization.id;
        let runners =
            tokio::task::spawn_blocking(move || store.runners(workspace, organization_id))
                .await
                .internal()?
                .internal()?;
        if !retirement_results(&input, organization_id, &runners)
            .iter()
            .all(|result| matches!(result, RunnerRetirement::Busy | RunnerRetirement::Retired))
        {
            return Err(Error::StateChanged);
        }
        for intent in input
            .overflow_intents
            .iter()
            .filter(|intent| intent.organization_id == organization_id)
        {
            if !self
                .current_overflow_busy(workspace, intent.runner_id)
                .await?
            {
                return Err(Error::StateChanged);
            }
        }
        Ok(())
    }
}
