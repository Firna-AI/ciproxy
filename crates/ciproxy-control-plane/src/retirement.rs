//! Durable stop intents share one delivery path across live work and recovery.

use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_store_interface::own_runners::{
    OwnRunner, OwnRunnerLease, OwnRunnerObservation, RoutingSnapshot,
};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

/// Positive evidence required before the durable stop write.
pub(crate) enum StopAuthority {
    /// A successful scoped GitHub removal or live absence query.
    GitHubConfirmed { github_id: Option<i64> },
    /// The owner lifecycle step validated the exact removed installation.
    OwnerRemoval,
}

impl MachineRuntime {
    /// Save a GitHub-first retirement and deliver its new stop intent once.
    pub(crate) async fn stop(&self, input: &RoutingSnapshot, runner: &OwnRunner) -> Result<()> {
        let Some(stopping) = self.retire_runner(input.workspace_id, runner).await? else {
            return Ok(());
        };
        let organization = input
            .organizations
            .iter()
            .find(|organization| organization.id == runner.runner.organization_id)
            .ok_or(Error::StateChanged)?;
        self.deliver_stop(input.workspace_id, organization.github_id, &stopping)
            .await
    }

    /// Persist a fresh stop under the caller's approved retirement policy.
    pub(crate) async fn retire_runner(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
    ) -> Result<Option<OwnRunner>> {
        if !matches!(runner.runner.place, RunnerPlace::Machine { .. }) {
            return Ok(None);
        }
        if matches!(
            runner.runner.state,
            RunnerState::Stopped | RunnerState::Stopping
        ) {
            return Ok(None);
        }
        if runner.runner.state == RunnerState::Busy {
            return Ok(None);
        }
        let target = ListenerTarget {
            workspace_id,
            organization_id: runner.runner.organization_id,
            scale_set_id: runner.runner.scale_set_id,
        };
        let authority = match self
            .registrations
            .retire(
                target,
                &format!("ciproxy-{}", runner.runner.id),
                runner.runner.github_id,
            )
            .await
            .internal()?
        {
            RegistrationRetirement::Removed { id } => StopAuthority::GitHubConfirmed {
                github_id: Some(id),
            },
            RegistrationRetirement::Absent => StopAuthority::GitHubConfirmed { github_id: None },
            RegistrationRetirement::Busy { id } => {
                self.observe_runner(
                    workspace_id,
                    runner,
                    OwnRunnerObservation {
                        state: RunnerState::Busy,
                        github_id: Some(id),
                        observed_at: self.clock.now(),
                    },
                )
                .await?;
                return Ok(None);
            }
        };
        Ok(Some(self.save_stop(workspace_id, runner, authority).await?))
    }

    /// Persist the GitHub identity and stop command only after positive authority.
    pub(crate) async fn save_stop(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
        authority: StopAuthority,
    ) -> Result<OwnRunner> {
        let github_id = match authority {
            StopAuthority::GitHubConfirmed { github_id } => github_id,
            StopAuthority::OwnerRemoval => runner.runner.github_id,
        };
        let mut stopping = runner.clone();
        stopping.command_id = self.clock.uuid();
        stopping.issued_at = self.clock.now();
        stopping.acknowledged = false;
        stopping.runner.state = RunnerState::Stopping;
        stopping.runner.github_id = github_id;
        let store = self.store.clone();
        let saved = stopping.clone();
        tokio::task::spawn_blocking(move || {
            store.retire_own_runner(
                workspace_id,
                saved.runner.id,
                saved.command_id,
                saved.issued_at,
                github_id,
            )
        })
        .await
        .internal()?
        .internal()?;
        Ok(stopping)
    }

    /// Retry pending stops before routing while checking the workspace lease.
    pub(crate) async fn recover_stops(
        &self,
        input: &RoutingSnapshot,
        lease: &mut dyn OwnRunnerLease,
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
            lease.check_connection().internal()?;
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
        let message = ControlMessage::StopRunner {
            command_id: CommandId(runner.command_id.to_string()),
            organization_id: OrganizationId(github_organization_id.to_string()),
            runner_id: RunnerId(runner.runner.id.to_string()),
        };
        match tokio::task::spawn_blocking(move || gateway.send(workspace_id, machine_id, message))
            .await
            .internal()?
        {
            Ok(_)
            | Err(ciproxy_gateway::error::Error::NotFound)
            | Err(ciproxy_gateway::error::Error::Unauthorized) => Ok(()),
            Err(error) => Err(error).internal(),
        }
    }
}
