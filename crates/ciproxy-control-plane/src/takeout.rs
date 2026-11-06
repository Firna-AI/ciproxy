//! Take-out acceptance keeps GitHub access until each reservation has an answer.

use ciproxy_github::error;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::own_runners::{OwnRunnerLease, RoutingSnapshot};
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    /// Accept only a matching take-out whose idle reservations have retirement proof.
    pub(crate) async fn takeout(&self, organization: &Organization) -> Result<()> {
        let workspace_id = organization.workspace_id;
        let ordinary = {
            let store = self.store.clone();
            let Some(mut lease) =
                tokio::task::spawn_blocking(move || store.try_lock_own_runners(workspace_id))
                    .await
                    .internal()?
                    .internal()?
            else {
                return Err(Error::StateChanged);
            };
            let input = self.load(workspace_id).await?;
            let matching = input.organizations.iter().any(|current| {
                current.id == organization.id
                    && current.github_id == organization.github_id
                    && current.installation_id == organization.installation_id
                    && current.app_state == AppState::Removed
            });
            if matching {
                match self.installations.by_id(organization.installation_id).await {
                    Ok(_) => {
                        self.retire_takeout_set(&input, organization, &mut *lease)
                            .await?
                    }
                    Err(error::Error::InstallationPending) => {
                        self.retire_owner_set(&input, organization, &mut *lease)
                            .await?
                    }
                    Err(error) => return Err(error).internal(),
                }
                false
            } else {
                true
            }
        };
        if ordinary {
            self.reconcile_workspace(workspace_id).await
        } else {
            Ok(())
        }
    }

    async fn retire_takeout_set(
        &self,
        input: &RoutingSnapshot,
        organization: &Organization,
        lease: &mut dyn OwnRunnerLease,
    ) -> Result<()> {
        let workspace_id = input.workspace_id;
        let mut failed = false;
        for runner in input
            .runners
            .iter()
            .filter(|runner| runner.runner.organization_id == organization.id)
        {
            if matches!(
                runner.runner.state,
                RunnerState::Busy | RunnerState::Stopped
            ) {
                continue;
            }
            lease.check_connection().internal()?;
            let stopping = if runner.runner.state == RunnerState::Stopping {
                runner.clone()
            } else {
                match self.retire_runner(workspace_id, runner).await {
                    Ok(Some(stopping)) => stopping,
                    Ok(None) => continue,
                    Err(error) => {
                        failed = true;
                        tracing::warn!(runner=%runner.runner.id,%error,"take-out retirement lacks authority; independent runners continue");
                        continue;
                    }
                }
            };
            lease.check_connection().internal()?;
            if !stopping.acknowledged
                && let Err(error) = self
                    .deliver_stop(workspace_id, organization.github_id, &stopping)
                    .await
            {
                tracing::warn!(runner=%runner.runner.id,%error,"authorized take-out stop delivery will retry");
            }
        }
        let saved = self.load(workspace_id).await?;
        let unanswered = saved.runners.iter().any(|runner| {
            runner.runner.organization_id == organization.id
                && !matches!(
                    runner.runner.state,
                    RunnerState::Busy | RunnerState::Stopping | RunnerState::Stopped
                )
        });
        if failed || unanswered {
            Err(Error::StateChanged)
        } else {
            Ok(())
        }
    }
}
