//! Take-out acceptance keeps GitHub access until each reservation has an answer.

use ciproxy_github::error;
use ciproxy_github::persistence::lease::{Lease, LeaseStore};
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{Error, ErrorResultExt, Result};
use crate::retirement::RunnerRetirement;
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

pub(crate) struct TakeoutCheck {
    pub(crate) organization: Organization,
    pub(crate) owner_removed: bool,
    pub(crate) generation: u64,
    pub(crate) failed: bool,
}

impl MachineRuntime {
    /// Accept only a matching take-out whose idle reservations have retirement proof.
    pub(crate) async fn takeout(&self, organization: &Organization) -> Result<()> {
        let workspace_id = organization.workspace_id;
        let input = self.load(workspace_id).await?;
        let matching = input.organizations.iter().any(|current| {
            current.id == organization.id
                && current.github_id == organization.github_id
                && current.installation_id == organization.installation_id
                && current.app_state == AppState::Removed
        });
        if !matching {
            return self.reconcile_workspace(workspace_id).await;
        }
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Err(Error::StateChanged);
        };
        lease.check().await.internal()?;
        let input = self.load(workspace_id).await?;
        let current = input
            .organizations
            .iter()
            .find(|current| current.id == organization.id)
            .cloned()
            .filter(|current| {
                current.installation_id == organization.installation_id
                    && current.app_state == AppState::Removed
            })
            .ok_or(Error::StateChanged)?;
        let checked = self
            .takeout_checks
            .lock()
            .await
            .get(&organization.id)
            .filter(|check| check.organization == current)
            .map(|check| check.owner_removed);
        if let Some(owner_removed) = checked {
            let result = if owner_removed {
                self.retire_owner_set(&input, organization, &*lease).await
            } else {
                self.retire_takeout_set(&input, organization, &*lease).await
            };
            if !owner_removed && result.is_err() {
                lease.check().await.internal()?;
                self.check_takeout_installation(organization, current, false)?;
            }
            return result;
        }
        lease.check().await.internal()?;
        self.check_takeout_installation(organization, current, true)?;
        Err(Error::StateChanged)
    }

    /// A positive answer serves one pending pass without replacing its Busy generation.
    fn check_takeout_installation(
        &self,
        organization: &Organization,
        current: Organization,
        initialize: bool,
    ) -> Result<()> {
        let workspace_id = organization.workspace_id;
        let runtime = self.worker();
        let organization = organization.clone();
        self.dispatch(Work::Takeout(organization.id), async move {
            let owner_removed = match runtime
                .installations
                .by_id(organization.installation_id)
                .await
            {
                Ok(_) => false,
                Err(error::Error::InstallationPending) => true,
                Err(error) => return Err(error).internal(),
            };
            let _operation = runtime.operation.lock().await;
            let Some(lease) = runtime
                .store
                .own_runner_lease(workspace_id)
                .await
                .internal()?
            else {
                return Err(Error::StateChanged);
            };
            let input = runtime.load(workspace_id).await?;
            if !input.organizations.iter().any(|current| {
                current.id == organization.id
                    && current.github_id == organization.github_id
                    && current.installation_id == organization.installation_id
                    && current.app_state == AppState::Removed
            }) {
                return Ok(());
            }
            let saved = input
                .organizations
                .iter()
                .find(|saved| saved.id == organization.id)
                .cloned()
                .ok_or(Error::StateChanged)?;
            if saved != current {
                return Ok(());
            }
            {
                let mut checks = runtime.takeout_checks.lock().await;
                if let Some(check) = checks
                    .get_mut(&organization.id)
                    .filter(|check| check.organization == saved)
                {
                    check.owner_removed |= owner_removed;
                } else {
                    checks.insert(
                        organization.id,
                        TakeoutCheck {
                            organization: saved,
                            owner_removed,
                            generation: 0,
                            failed: false,
                        },
                    );
                }
            }
            if !initialize {
                return Ok(());
            }
            if owner_removed {
                return runtime
                    .retire_owner_set(&input, &organization, &*lease)
                    .await;
            }
            match runtime
                .retire_takeout_set(&input, &organization, &*lease)
                .await
            {
                Ok(()) | Err(Error::StateChanged) => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    async fn retire_takeout_set(
        &self,
        input: &RoutingSnapshot,
        organization: &Organization,
        lease: &dyn Lease,
    ) -> Result<()> {
        let workspace_id = input.workspace_id;
        let reset = {
            let mut checks = self.takeout_checks.lock().await;
            let check = checks
                .get_mut(&organization.id)
                .ok_or(Error::StateChanged)?;
            if check.failed {
                check.failed = false;
                check.generation = check.generation.wrapping_add(1);
                true
            } else {
                false
            }
        };
        if reset {
            self.overflow_answers
                .lock()
                .await
                .retain(|_, (_, guard)| guard.intent.organization_id != organization.id);
        }
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
            lease.check().await.internal()?;
            let stopping = if runner.runner.state == RunnerState::Stopping {
                runner.clone()
            } else {
                match self.retire_runner(workspace_id, runner).await {
                    Ok(_) => continue,
                    Err(error) => {
                        failed = true;
                        tracing::warn!(runner=%runner.runner.id,%error,"take-out retirement lacks authority; independent runners continue");
                        continue;
                    }
                }
            };
            lease.check().await.internal()?;
            if !stopping.acknowledged
                && let Err(error) = self
                    .deliver_stop(workspace_id, organization.github_id, &stopping)
                    .await
            {
                tracing::warn!(runner=%runner.runner.id,%error,"authorized take-out stop delivery will retry");
            }
        }
        lease.check().await.internal()?;
        if let Err(error) = self.cleanup_overflow(workspace_id).await {
            tracing::warn!(%error,"take-out overflow cleanup will retry from saved retirement");
        }
        let current = self.load(workspace_id).await?;
        let results = self
            .retire_overflow_registrations(&current, organization.id, lease)
            .await?;
        if failed
            || results.iter().any(|result| {
                matches!(
                    result,
                    RunnerRetirement::Pending | RunnerRetirement::Deferred
                )
            })
        {
            return Err(Error::StateChanged);
        }
        self.takeout_accepted(organization).await
    }
}
