//! GitHub-first idle retirement with the App-removal lifetime exception.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrations};
use ciproxy_github::runtime::Runtime;
use ciproxy_overflow::api::{Retirement, RunnerAuthority};
use ciproxy_overflow::error::{Error, Result};
use ciproxy_overflow::view::OverflowView;
use ciproxy_scaler::actions::Action;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

/// Shared registration authority used by overflow cleanup and runtime recovery.
pub struct BoatRunnerAuthority {
    /// Current released Boat image requirement.
    pub root_sizing: RootSizing,
    /// Shared runtime clock.
    pub clock: Arc<dyn Runtime>,
    /// Current workspace and installation facts.
    pub store: Arc<dyn Store>,
    /// Live GitHub registration removal with busy protection.
    pub registrations: Arc<dyn RunnerRegistrations>,
}

/// GitHub retirement shared by startup recovery, take-out, and overflow maintenance.
#[async_trait]
pub trait BoatRegistrationRetirement: Send + Sync {
    /// Obtain live GitHub removal or Busy proof. Callers enforce delivery safety.
    async fn remove(&self, workspace_id: Uuid, intent: &BoatRunnerIntent) -> Result<Retirement>;
}

#[async_trait]
impl RunnerAuthority for BoatRunnerAuthority {
    async fn retire(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Retirement> {
        let store = self.store.clone();
        let read = tokio::task::spawn_blocking(move || {
            let input = store.own_routing_snapshot(workspace_id)?;
            let mut runnable = Vec::new();
            for org in &input.organizations {
                if store.organization_runnable(workspace_id, org.id)? {
                    runnable.push(org.id);
                }
            }
            Ok::<_, ciproxy_store_interface::error::Error>((input, runnable))
        })
        .await;
        let (input, runnable) = match read {
            Ok(Ok(value)) => value,
            _ => return Err(Error::ProviderUnavailable),
        };
        if input.organizations.iter().any(|organization| {
            organization.id == organization_id && organization.removed_at.is_some()
        }) {
            return Err(Error::RunnerNeeded);
        }
        let intents = &input.boat_intents;
        let intent = intents
            .iter()
            .find(|intent| {
                intent.runner_id == runner_id && intent.organization_id == organization_id
            })
            .ok_or(Error::OwnershipUnknown)?;
        let takeout = input.organizations.iter().any(|organization| {
            organization.id == organization_id && organization.app_state == AppState::Removed
        });
        if !takeout
            && input.jobs.iter().any(|job| {
                job.organization_id == organization_id
                    && job.scale_set_id == intent.scale_set_id
                    && job.phase == DemandPhase::Started
                    && job.runner_github_id.is_some()
                    && job.runner_github_id == intent.github_id
            })
        {
            return Ok(Retirement::RunningJob);
        }
        let view = OverflowView::from_snapshot(input.overflow.clone(), self.clock.now())?;
        let mut snapshot =
            crate::snapshot::build(&input, &runnable, self.clock.now(), self.root_sizing);
        snapshot.boat = view.retained_scaler_account()?;
        crate::boat_snapshot::append(&mut snapshot, intents, &view.starts, &input.jobs);
        let sandbox = snapshot.sandboxes.iter().find(|sandbox| {
            sandbox
                .runner
                .as_ref()
                .is_some_and(|retained| retained.runner.id.0 == runner_id.to_string())
        });
        let retire = sandbox.is_none_or(|sandbox| {
            ciproxy_scaler::decision::decide(&snapshot).iter().any(|action|
            matches!(action, Action::DeleteSandbox {sandbox_id,..} if *sandbox_id == sandbox.id))
        });
        if !takeout && !retire {
            return Ok(Retirement::WaitingDemand);
        }
        let retirement = self.remove(workspace_id, intent).await?;
        if retirement == Retirement::Removed {
            let store = self.store.clone();
            match tokio::task::spawn_blocking(move || {
                store.organization(workspace_id, organization_id)
            })
            .await
            {
                Ok(Ok(Some(organization))) if organization.removed_at.is_none() => {}
                Ok(Ok(Some(_))) => return Err(Error::RunnerNeeded),
                _ => return Err(Error::ProviderUnavailable),
            }
        }
        Ok(retirement)
    }
}

#[async_trait]
impl BoatRegistrationRetirement for BoatRunnerAuthority {
    async fn remove(&self, workspace_id: Uuid, intent: &BoatRunnerIntent) -> Result<Retirement> {
        let result = self
            .registrations
            .retire(
                ListenerTarget {
                    workspace_id,
                    organization_id: intent.organization_id,
                    scale_set_id: intent.scale_set_id,
                },
                &format!("ciproxy-{}", intent.runner_id),
                intent.github_id,
            )
            .await;
        match result {
            Ok(RegistrationRetirement::Busy { id }) => {
                let store = self.store.clone();
                let runner = intent.runner_id;
                let organization = intent.organization_id;
                match tokio::task::spawn_blocking(move || {
                    store.register_boat_runner(workspace_id, runner, id)?;
                    let input = store.boat_runtime_snapshot(workspace_id)?;
                    if input.starts.iter().any(|record| {
                        record.runner_id == runner
                            && record.phase == StartPhase::Running
                            && record.ends_at.is_none()
                    }) {
                        store.update_runner(
                            workspace_id,
                            organization,
                            runner,
                            RunnerState::Busy,
                            Some(id),
                        )?;
                    }
                    Ok::<_, ciproxy_store_interface::error::Error>(())
                })
                .await
                {
                    Ok(Ok(())) => Ok(Retirement::RunningJob),
                    _ => Err(Error::ProviderUnavailable),
                }
            }
            Ok(RegistrationRetirement::Removed { .. } | RegistrationRetirement::Absent) => {
                Ok(Retirement::Removed)
            }
            Err(_) => Err(Error::ProviderUnavailable),
        }
    }
}
