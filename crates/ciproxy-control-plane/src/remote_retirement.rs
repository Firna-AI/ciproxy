//! Each GitHub retirement waits outside reconciliation and event processing.

use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::runtime_results::{BoatRunnerResult, OwnRunnerGuard, OwnRunnerResult};
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn dispatch_retirement(
        &self,
        workspace: Uuid,
        runner: &OwnRunner,
        orphan: bool,
    ) -> Result<()> {
        if self.task_scope()?.contains(Work::Create(runner.runner.id)) {
            return Ok(());
        }
        let guard = self.runner_guard(workspace, runner).await?;
        let runtime = self.worker();
        self.dispatch(Work::Retire(runner.runner.id), async move {
            match runtime.remote_retirement(workspace, guard, orphan).await {
                Ok(()) => Ok(()),
                Err(error) => {
                    tracing::warn!(%error,"registration retirement remains pending");
                    Err(Error::StateChanged)
                }
            }
        })
    }

    async fn remote_retirement(
        &self,
        workspace: Uuid,
        expected: OwnRunnerGuard,
        orphan: bool,
    ) -> Result<()> {
        let runner = &expected.runner;
        let target = ListenerTarget {
            workspace_id: workspace,
            organization_id: runner.runner.organization_id,
            scale_set_id: runner.runner.scale_set_id,
        };
        let answer = self
            .registrations
            .retire(
                target,
                &format!("ciproxy-{}", runner.runner.id),
                runner.runner.github_id,
            )
            .await
            .internal()?;
        let github_id = match answer {
            RegistrationRetirement::Busy { id } => {
                self.apply_remote(
                    workspace,
                    expected,
                    OwnRunnerResult::Busy {
                        github_id: id,
                        at: self.clock.now(),
                    },
                )
                .await?;
                return Ok(());
            }
            RegistrationRetirement::Removed { id } => Some(id),
            RegistrationRetirement::Absent => {
                if orphan {
                    runner.runner.github_id
                } else {
                    None
                }
            }
        };
        if orphan || runner.runner.state == RunnerState::Stopped {
            self.apply_remote(workspace, expected, OwnRunnerResult::Stopped { github_id })
                .await?;
            return Ok(());
        }
        let mut stopping = runner.clone();
        stopping.command_id = self.clock.uuid();
        stopping.issued_at = self.clock.now();
        stopping.acknowledged = false;
        stopping.runner.state = RunnerState::Stopping;
        stopping.runner.github_id = github_id;
        let organization = expected
            .organization
            .as_ref()
            .map(|organization| organization.github_id);
        let update = OwnRunnerResult::Retired {
            command_id: stopping.command_id,
            at: stopping.issued_at,
            github_id,
        };
        if self.apply_remote(workspace, expected, update).await?
            && let Some(organization) = organization
        {
            self.deliver_stop(workspace, organization, &stopping)
                .await?;
        }
        Ok(())
    }

    /// A late JIT identity must remain owned even when its original start lost authority.
    pub(crate) fn cleanup_late_registration(
        &self,
        workspace: Uuid,
        runner: Uuid,
        target: RegistrationTarget,
        github_id: i64,
    ) -> Result<()> {
        let runtime = self.worker();
        self.dispatch(Work::Retire(runner), async move {
            let store = runtime.store.clone();
            let expected =
                tokio::task::spawn_blocking(move || store.own_runner_guard(workspace, runner))
                    .await
                    .internal()?
                    .internal()?;
            let store = runtime.store.clone();
            let boat =
                tokio::task::spawn_blocking(move || store.boat_runner_guard(workspace, runner))
                    .await
                    .internal()?
                    .internal()?;
            let answer = runtime
                .registrations
                .retire(
                    ListenerTarget {
                        workspace_id: workspace,
                        organization_id: target.organization_id,
                        scale_set_id: target.scale_set_id,
                    },
                    &format!("ciproxy-{runner}"),
                    Some(github_id),
                )
                .await
                .internal()?;
            match answer {
                RegistrationRetirement::Busy { id } => {
                    if let Some(expected) = expected
                        && expected.runner.runner.state != RunnerState::Stopped
                    {
                        runtime
                            .apply_remote(
                                workspace,
                                expected,
                                OwnRunnerResult::Busy {
                                    github_id: id,
                                    at: runtime.clock.now(),
                                },
                            )
                            .await?;
                    }
                }
                RegistrationRetirement::Removed { .. } | RegistrationRetirement::Absent => {
                    if let Some(expected) = boat {
                        let store = runtime.store.clone();
                        tokio::task::spawn_blocking(move || {
                            store.apply_boat_runner_result(
                                workspace,
                                &expected,
                                BoatRunnerResult::Retired,
                            )
                        })
                        .await
                        .internal()?
                        .internal()?;
                    }
                }
            }
            Ok(())
        })
    }
}
