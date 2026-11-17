//! Reconcile authenticated agent observations with durable GitHub identities.

use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation, RoutingSnapshot};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn agent_event(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: AgentMessage,
    ) -> Result<()> {
        if let AgentMessage::CommandResult {
            command_id,
            outcome,
        } = &message
        {
            let image_command = {
                let mut requests = self.image_requests.lock().await;
                let key = (workspace_id, machine_id);
                let matched = requests
                    .get(&key)
                    .is_some_and(|request| request.command_id.to_string() == command_id.0);
                if matched {
                    requests.remove(&key);
                }
                matched
            };
            if image_command {
                return Ok(());
            }
            let Ok(command) = command_id.0.parse() else {
                return Ok(());
            };
            let store = self.store.clone();
            let Some(runner) =
                tokio::task::spawn_blocking(move || store.own_command(workspace_id, command))
                    .await
                    .internal()?
                    .internal()?
            else {
                return Ok(());
            };
            require_machine(&runner, machine_id)?;
            let store = self.store.clone();
            let command = runner.command_id;
            let applied = *outcome == CommandOutcome::Applied;
            let at = self.clock.now();
            tokio::task::spawn_blocking(move || {
                store.record_own_command_result(workspace_id, command, applied, at)
            })
            .await
            .internal()?
            .internal()?;
            if !applied && runner.runner.state == RunnerState::Starting {
                self.dispatch_retirement(workspace_id, &runner, false)
                    .await?;
            }
            return Ok(());
        }
        let input = self.load(workspace_id).await?;
        let (id, state) = match message {
            AgentMessage::RunnerReady { runner_id } => (runner_id, RunnerState::Ready),
            AgentMessage::JobStarted { runner_id, .. } => (runner_id, RunnerState::Busy),
            AgentMessage::RunnerExited { runner_id, .. } => (runner_id, RunnerState::Stopped),
            _ => return Ok(()),
        };
        let runner = input
            .runners
            .iter()
            .find(|runner| runner.runner.id.to_string() == id.0)
            .ok_or(Error::RunnerOwnership)?;
        require_machine(runner, machine_id)?;
        if runner.runner.state == RunnerState::Stopped && state != RunnerState::Stopped {
            return Ok(());
        }
        let protected_ready = state == RunnerState::Ready
            && matches!(
                runner.runner.state,
                RunnerState::Busy | RunnerState::Stopping
            );
        self.observe_runner(
            workspace_id,
            runner,
            OwnRunnerObservation {
                state: if protected_ready {
                    runner.runner.state
                } else {
                    state
                },
                github_id: runner.runner.github_id,
                observed_at: self.clock.now(),
            },
        )
        .await?;
        if protected_ready {
            return Ok(());
        }
        let input = self.load(workspace_id).await?;
        for job in &input.jobs {
            self.route_job(&input, job).await?;
        }
        Ok(())
    }

    pub(crate) async fn route_job(&self, input: &RoutingSnapshot, job: &GithubJob) -> Result<()> {
        let workspace_id = input.workspace_id;
        if job.phase < DemandPhase::Started {
            return Ok(());
        }
        let Some(github_id) = job.runner_github_id else {
            return Ok(());
        };
        let Some(runner) = input.runners.iter().find(|runner| {
            runner.runner.github_id == Some(github_id)
                && runner.runner.organization_id == job.organization_id
                && runner.runner.scale_set_id == job.scale_set_id
        }) else {
            return Ok(());
        };
        if runner.runner.state == RunnerState::Stopped {
            return Ok(());
        }
        let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
            return Ok(());
        };
        if job.phase == DemandPhase::Completed
            && !input
                .machines
                .iter()
                .any(|machine| machine.machine.id == machine_id)
        {
            if input.organizations.iter().any(|organization| {
                organization.id == job.organization_id && organization.app_state == AppState::Active
            }) {
                self.dispatch_retirement(workspace_id, runner, true).await?;
            }
            return Ok(());
        }
        if job.phase == DemandPhase::Started && runner.runner.state != RunnerState::Stopping {
            self.observe_runner(
                workspace_id,
                runner,
                OwnRunnerObservation {
                    state: RunnerState::Busy,
                    github_id: Some(github_id),
                    observed_at: self.clock.now(),
                },
            )
            .await?;
        }
        self.prepare_job(input, runner, job, machine_id).await
    }
}

fn require_machine(runner: &OwnRunner, machine_id: Uuid) -> Result<()> {
    match runner.runner.place {
        RunnerPlace::Machine {
            machine_id: owner, ..
        } if owner == machine_id => Ok(()),
        _ => Err(Error::RunnerOwnership),
    }
}
