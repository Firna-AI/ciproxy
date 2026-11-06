//! Organization lifecycle events retain their exact retirement authority.

use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::own_runners::{OwnRunnerLease, RoutingSnapshot};
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{Error, ErrorResultExt, Result};
use crate::retirement::StopAuthority;
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    /// Retire local inactive idle work only while this exact installation stays inactive.
    pub(crate) async fn inactive(&self, organization: &Organization) -> Result<()> {
        let workspace_id = organization.workspace_id;
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
        let current = input.organizations.iter().any(|current| {
            current.id == organization.id
                && current.github_id == organization.github_id
                && current.installation_id == organization.installation_id
                && current.app_state == AppState::Active
        });
        if !current
            || self
                .runnable_organizations(&input)
                .await?
                .contains(&organization.id)
        {
            return Ok(());
        }
        for job in input
            .jobs
            .iter()
            .filter(|job| job.organization_id == organization.id)
        {
            self.route_job(&input, job).await?;
        }
        let input = self.load(workspace_id).await?;
        let mut failed = false;
        for runner in input.runners.iter().filter(|runner| {
            runner.runner.organization_id == organization.id
                && runner.runner.state != RunnerState::Stopped
        }) {
            lease.check_connection().internal()?;
            if let Err(error) = self.stop(&input, runner).await {
                failed = true;
                tracing::warn!(runner=%runner.runner.id,%error,"inactive organization retirement will retry; independent runners continue");
            }
        }
        if failed {
            Err(Error::StateChanged)
        } else {
            Ok(())
        }
    }

    /// Apply an immutable owner removal, with conservative protection after reinstall.
    pub(crate) async fn owner_removed(&self, organization: &Organization) -> Result<()> {
        let workspace_id = organization.workspace_id;
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
        let immediate = input.organizations.iter().any(|current| {
            current.id == organization.id
                && current.github_id == organization.github_id
                && current.installation_id == organization.installation_id
                && current.app_state == AppState::Removed
        });
        if immediate {
            return self
                .retire_owner_set(&input, organization, &mut *lease)
                .await;
        }
        let removed_at = organization.removed_at.ok_or(Error::StateChanged)?;
        let mut failed = false;
        for runner in input.runners.iter().filter(|runner| {
            runner.runner.organization_id == organization.id
                && runner.runner.state != RunnerState::Stopped
                && runner.issued_at <= removed_at
        }) {
            lease.check_connection().internal()?;
            if let Err(error) = self.stop(&input, runner).await {
                failed = true;
                tracing::warn!(runner=%runner.runner.id,%error,"reinstalled organization retirement will retry");
            }
        }
        if failed {
            Err(Error::StateChanged)
        } else {
            Ok(())
        }
    }

    /// Save immediate stops only after exact owner-removal authority is established.
    pub(crate) async fn retire_owner_set(
        &self,
        input: &RoutingSnapshot,
        organization: &Organization,
        lease: &mut dyn OwnRunnerLease,
    ) -> Result<()> {
        let workspace_id = input.workspace_id;
        let mut failed = false;
        for runner in input.runners.iter().filter(|runner| {
            runner.runner.organization_id == organization.id
                && runner.runner.state != RunnerState::Stopped
        }) {
            lease.check_connection().internal()?;
            let stopping = if runner.runner.state == RunnerState::Stopping {
                runner.clone()
            } else {
                match self
                    .save_stop(workspace_id, runner, StopAuthority::OwnerRemoval)
                    .await
                {
                    Ok(stopping) => stopping,
                    Err(error) => {
                        failed = true;
                        tracing::warn!(runner=%runner.runner.id,%error,"owner removal intent will retry; independent runners continue");
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
                tracing::warn!(runner=%runner.runner.id,%error,"saved owner removal stop will retry");
            }
        }
        if failed {
            Err(Error::StateChanged)
        } else {
            Ok(())
        }
    }
}
