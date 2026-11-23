//! GitHub-first idle retirement with saved installation ownership holds.

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
use ciproxy_scaler::decision::decide;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_runtime::OverflowRunnerIntent;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::runtime_results::{
    OverflowRunnerGuard, OverflowRunnerResult, OverflowRunnerWrite,
};
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use crate::boat_snapshot;
use crate::snapshot::build;

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

/// A live retirement result tied to the saved authority that accepted it.
pub struct BoatRemoval {
    /// GitHub's current retirement decision.
    pub retirement: Retirement,
    /// Exact saved authority after the conditional result write.
    pub guard: OverflowRunnerGuard,
}

/// GitHub retirement shared by startup recovery, take-out, and overflow maintenance.
#[async_trait]
pub trait BoatRegistrationRetirement: Send + Sync {
    /// Obtain live GitHub removal or Busy proof. Callers enforce delivery safety.
    async fn remove(
        &self,
        workspace_id: Uuid,
        intent: &OverflowRunnerIntent,
    ) -> Result<BoatRemoval>;
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
        let intents = &input.overflow_intents;
        let intent = intents
            .iter()
            .find(|intent| {
                intent.runner_id == runner_id && intent.organization_id == organization_id
            })
            .ok_or(Error::OwnershipUnknown)?;
        if self
            .registration_access_ended(workspace_id, runner_id)
            .await?
        {
            return Err(Error::RunnerNeeded);
        }
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
        let busy = input.overflow_runners.iter().any(|runner| {
            runner.id == runner_id
                && runner.organization_id == organization_id
                && runner.state == RunnerState::Busy
        });
        if !takeout && !busy {
            let view = OverflowView::from_snapshot(input.overflow.clone(), self.clock.now())?;
            let boat = view
                .provider(OverflowProvider::Boat)
                .ok_or(Error::ProviderUnavailable)?;
            let mut snapshot = build(&input, &runnable, self.clock.now(), self.root_sizing);
            snapshot.providers.push(boat_snapshot::account(
                self.root_sizing,
                boat.retained_scaler_account()?,
            ));
            let sandbox = snapshot.sandboxes.iter().find(|sandbox| {
                sandbox
                    .runner
                    .as_ref()
                    .is_some_and(|retained| retained.runner.id.0 == runner_id.to_string())
            });
            let retire = sandbox.is_none_or(|sandbox| {
                decide(&snapshot).iter().any(|action| {
                    matches!(action, Action::DeleteSandbox { provider, sandbox_id, .. }
                    if *provider == sandbox.provider && *sandbox_id == sandbox.id)
                })
            });
            if !retire {
                return Ok(Retirement::WaitingDemand);
            }
        }
        let retirement = self.remove(workspace_id, intent).await?.retirement;
        if retirement == Retirement::Removed
            && self
                .registration_access_ended(workspace_id, runner_id)
                .await?
        {
            return Err(Error::RunnerNeeded);
        }
        Ok(retirement)
    }
}

#[async_trait]
impl BoatRegistrationRetirement for BoatRunnerAuthority {
    async fn remove(
        &self,
        workspace_id: Uuid,
        intent: &OverflowRunnerIntent,
    ) -> Result<BoatRemoval> {
        if self
            .registration_access_ended(workspace_id, intent.runner_id)
            .await?
        {
            return Err(Error::RunnerNeeded);
        }
        let store = self.store.clone();
        let runner = intent.runner_id;
        let expected = match tokio::task::spawn_blocking(move || {
            store.overflow_runner_guard(workspace_id, runner)
        })
        .await
        {
            Ok(Ok(Some(expected))) => expected,
            _ => return Err(Error::OwnershipUnknown),
        };
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
                match tokio::task::spawn_blocking(move || {
                    store.apply_overflow_runner_result(
                        workspace_id,
                        &expected,
                        OverflowRunnerResult::Busy { github_id: id },
                    )
                })
                .await
                {
                    Ok(Ok(OverflowRunnerWrite::Applied(saved))) => Ok(BoatRemoval {
                        retirement: Retirement::RunningJob,
                        guard: *saved,
                    }),
                    Ok(Ok(OverflowRunnerWrite::Rejected | OverflowRunnerWrite::Retired)) => {
                        Err(Error::RunnerNeeded)
                    }
                    _ => Err(Error::ProviderUnavailable),
                }
            }
            Ok(RegistrationRetirement::Removed { .. } | RegistrationRetirement::Absent) => {
                let store = self.store.clone();
                match tokio::task::spawn_blocking(move || {
                    store.apply_overflow_runner_result(
                        workspace_id,
                        &expected,
                        OverflowRunnerResult::Checked,
                    )
                })
                .await
                {
                    Ok(Ok(OverflowRunnerWrite::Applied(saved))) => Ok(BoatRemoval {
                        retirement: Retirement::Removed,
                        guard: *saved,
                    }),
                    Ok(Ok(OverflowRunnerWrite::Rejected | OverflowRunnerWrite::Retired)) => {
                        Err(Error::RunnerNeeded)
                    }
                    _ => Err(Error::ProviderUnavailable),
                }
            }
            Err(_) => Err(Error::ProviderUnavailable),
        }
    }
}

impl BoatRunnerAuthority {
    async fn registration_access_ended(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<bool> {
        let store = self.store.clone();
        match tokio::task::spawn_blocking(move || {
            store.overflow_registration_access_ended(workspace_id, runner_id)
        })
        .await
        {
            Ok(Ok(ended)) => Ok(ended),
            _ => Err(Error::ProviderUnavailable),
        }
    }
}
