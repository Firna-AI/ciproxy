//! Take-out acceptance keeps GitHub access until each reservation has an answer.

use ciproxy_github::error;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::own_runners::{OwnRunnerLease, RoutingSnapshot};
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{Error, ErrorResultExt, Result};
use crate::reservations::retirement_results;
use crate::retirement::RunnerRetirement;
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
                    Ok(RunnerRetirement::Stopping(stopping)) => *stopping,
                    Ok(_) => continue,
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
        lease.check_connection().internal()?;
        if let Err(error) = self.cleanup_boat(workspace_id).await {
            tracing::warn!(%error,"take-out Boat cleanup will retry from saved retirement");
        }
        let current = self.load(workspace_id).await?;
        let mut results = self
            .retire_boat_registrations(&current, organization.id, lease)
            .await?;
        let saved = self.load(workspace_id).await?;
        let store = self.store.clone();
        let organization_id = organization.id;
        let runners =
            tokio::task::spawn_blocking(move || store.runners(workspace_id, organization_id))
                .await
                .internal()?
                .internal()?;
        results.extend(retirement_results(&saved, organization.id, &runners));
        let unanswered = results.iter().any(|result| match result {
            RunnerRetirement::Pending | RunnerRetirement::Deferred => true,
            RunnerRetirement::Stopping(_) | RunnerRetirement::Retired | RunnerRetirement::Busy => {
                false
            }
        });
        if failed || unanswered {
            Err(Error::StateChanged)
        } else {
            Ok(())
        }
    }
}
