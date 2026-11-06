//! Immutable installation removals retain their approved retirement policy.

use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{Error, ErrorResultExt, Result};
use crate::retirement::RetirementPolicy;
use crate::runtime::MachineRuntime;

impl MachineRuntime {
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
        let removed_at = organization.removed_at.ok_or(Error::StateChanged)?;
        let mut failed = false;
        for runner in input.runners.iter().filter(|runner| {
            runner.runner.organization_id == organization.id
                && runner.runner.state != RunnerState::Stopped
                && (immediate || runner.issued_at <= removed_at)
        }) {
            lease.check_connection().internal()?;
            if !immediate {
                if let Err(error) = self.stop(&input, runner).await {
                    failed = true;
                    tracing::warn!(runner=%runner.runner.id,%error,"reinstalled organization retirement will retry");
                }
                continue;
            }
            let stopping = if runner.runner.state == RunnerState::Stopping {
                runner.clone()
            } else {
                match self
                    .retire_runner(workspace_id, runner, RetirementPolicy::OwnerRemoval)
                    .await
                {
                    Ok(Some(stopping)) => stopping,
                    Ok(None) => continue,
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
