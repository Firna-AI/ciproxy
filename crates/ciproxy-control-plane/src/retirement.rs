//! Durable stop intents share one delivery path across live work and recovery.

use ciproxy_github::persistence::lease::Lease;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_store_interface::own_runners::{OwnRunner, RoutingSnapshot};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::runtime_results::OwnRunnerResult;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

/// Saved authority shared by own-machine and overflow retirement acceptance.
pub(crate) enum RunnerRetirement {
    /// Durable retirement needs no further registration access.
    Retired,
    /// A saved Busy answer protects the running job.
    Busy,
    /// The reservation still lacks a durable answer.
    Pending,
    /// Owner removal or an absent organization prevents registration recovery.
    Deferred,
}

impl MachineRuntime {
    /// Save a GitHub-first retirement and deliver its new stop intent once.
    pub(crate) async fn stop(&self, input: &RoutingSnapshot, runner: &OwnRunner) -> Result<()> {
        self.retire_runner(input.workspace_id, runner).await?;
        Ok(())
    }

    /// Persist a fresh stop under the caller's approved retirement policy.
    pub(crate) async fn retire_runner(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
    ) -> Result<RunnerRetirement> {
        if !matches!(runner.runner.place, RunnerPlace::Machine { .. }) {
            return Ok(RunnerRetirement::Pending);
        }
        if matches!(
            runner.runner.state,
            RunnerState::Stopped | RunnerState::Stopping
        ) {
            return Ok(RunnerRetirement::Retired);
        }
        if runner.runner.state == RunnerState::Busy {
            return Ok(RunnerRetirement::Busy);
        }
        self.dispatch_retirement(workspace_id, runner, false)
            .await?;
        Ok(RunnerRetirement::Pending)
    }

    /// Persist the GitHub identity and stop command only after positive authority.
    pub(crate) async fn save_owner_stop(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
    ) -> Result<OwnRunner> {
        let github_id = runner.runner.github_id;
        let mut stopping = runner.clone();
        stopping.command_id = self.clock.uuid();
        stopping.issued_at = self.clock.now();
        stopping.acknowledged = false;
        stopping.runner.state = RunnerState::Stopping;
        stopping.runner.github_id = github_id;
        let expected = self.runner_guard(workspace_id, runner).await?;
        let store = self.store.clone();
        let update = OwnRunnerResult::Retired {
            command_id: stopping.command_id,
            at: stopping.issued_at,
            github_id,
        };
        let applied = tokio::task::spawn_blocking(move || {
            store.apply_own_runner_result(workspace_id, &expected, update)
        })
        .await
        .internal()?
        .internal()?;
        if !applied {
            return Err(Error::StateChanged);
        }
        Ok(stopping)
    }

    /// Retry pending stops before routing while checking the workspace lease.
    pub(crate) async fn recover_stops(
        &self,
        input: &RoutingSnapshot,
        lease: &dyn Lease,
    ) -> Result<()> {
        for runner in &input.runners {
            let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
                continue;
            };
            if runner.runner.state != RunnerState::Stopping
                || runner.acknowledged
                || !input
                    .machines
                    .iter()
                    .any(|machine| machine.machine.id == machine_id)
            {
                continue;
            }
            lease.check().await.internal()?;
            let Some(organization) = input
                .organizations
                .iter()
                .find(|organization| organization.id == runner.runner.organization_id)
            else {
                continue;
            };
            if let Err(error) = self
                .deliver_stop(input.workspace_id, organization.github_id, runner)
                .await
            {
                tracing::warn!(runner=%runner.runner.id,%error,"saved stop delivery will retry; independent runners continue");
            }
        }
        Ok(())
    }

    /// Send the persisted identity without repeating GitHub removal or changing it.
    pub(crate) async fn deliver_stop(
        &self,
        workspace_id: Uuid,
        github_organization_id: i64,
        runner: &OwnRunner,
    ) -> Result<()> {
        let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
            return Ok(());
        };
        let gateway = self.gateway.clone();
        let store = self.store.clone();
        let saved = runner.clone();
        let message = ControlMessage::StopRunner {
            command_id: CommandId(runner.command_id.to_string()),
            organization_id: OrganizationId(github_organization_id.to_string()),
            runner_id: RunnerId(runner.runner.id.to_string()),
        };
        tokio::task::spawn_blocking(move || {
            let current = store
                .own_command(workspace_id, saved.command_id)
                .internal()?;
            if current.is_none_or(|current| {
                current.runner.id != saved.runner.id
                    || current.runner.state != RunnerState::Stopping
            }) {
                return Ok(());
            }
            match gateway.send(workspace_id, machine_id, message) {
                Ok(_)
                | Err(ciproxy_gateway::error::Error::NotFound)
                | Err(ciproxy_gateway::error::Error::Unauthorized) => Ok(()),
                Err(error) => Err(error).internal(),
            }
        })
        .await
        .internal()?
    }
}
