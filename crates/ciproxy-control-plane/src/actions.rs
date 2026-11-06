//! Persist resource reservations before sending a one-use JIT to its machine.

use ciproxy_gateway::types::Delivery;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::Removal;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_protocol::runners::RunnerSpec;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Action, Destination, StartRunner};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::slots::allocate_slots;
use ciproxy_store_interface::own_runners::RoutingSnapshot;

impl MachineRuntime {
    pub(crate) async fn apply(&self, input: &RoutingSnapshot, action: Action) -> Result<()> {
        match action {
            Action::StartRunner(start) => self.start(input, start).await,
            Action::StopRunner { runner_id, .. } => {
                let runner = input
                    .runners
                    .iter()
                    .find(|runner| runner.runner.id.to_string() == runner_id.0)
                    .ok_or(Error::StateChanged)?;
                self.stop(input.workspace_id, runner).await
            }
            Action::DeleteSandbox { .. } => Ok(()),
        }
    }

    async fn start(&self, input: &RoutingSnapshot, start: StartRunner) -> Result<()> {
        let Destination::Machine(machine_id) = start.destination else {
            return Ok(());
        };
        let machine_id: Uuid = machine_id.0.parse().internal()?;
        let organization_id: Uuid = start.organization_id.0.parse().internal()?;
        let github_organization_id = input
            .organizations
            .iter()
            .find(|org| org.id == organization_id)
            .ok_or(Error::StateChanged)?
            .github_id;
        let label = input
            .labels
            .iter()
            .find(|label| label.name == start.label.0)
            .ok_or(Error::StateChanged)?;
        let store = self.store.clone();
        let workspace_id = input.workspace_id;
        let sets =
            tokio::task::spawn_blocking(move || store.scale_sets(workspace_id, organization_id))
                .await
                .internal()?
                .internal()?;
        let set = sets
            .iter()
            .find(|set| set.label_id == label.id)
            .ok_or(Error::StateChanged)?;
        let store = self.store.clone();
        let reservations = tokio::task::spawn_blocking(move || store.own_runners(workspace_id))
            .await
            .internal()?
            .internal()?;
        let capacity = input
            .machines
            .iter()
            .find(|machine| machine.machine.id == machine_id)
            .and_then(|machine| machine.report.as_ref())
            .map_or(0, |report| report.capacity.0);
        let slot_indices = allocate_slots(machine_id, start.size, capacity, &reservations)
            .ok_or(Error::StateChanged)?;
        let id = self.clock.uuid();
        let command_id = self.clock.uuid();
        let target = ListenerTarget {
            workspace_id,
            organization_id,
            scale_set_id: set.id,
        };
        let runner = OwnRunner {
            requested_size: label.size,
            slot_indices: slot_indices.clone(),
            runner: Runner {
                id,
                organization_id,
                label_id: label.id,
                scale_set_id: set.id,
                github_id: None,
                state: RunnerState::Starting,
                size: start.size,
                place: RunnerPlace::Machine {
                    machine_id,
                    first_slot: i32::try_from(*slot_indices.first().ok_or(Error::StateChanged)?)
                        .internal()?,
                },
            },
            label_name: label.name.clone(),
            command_id,
            issued_at: self.clock.now(),
            acknowledged: false,
        };
        let store = self.store.clone();
        let saved = runner.clone();
        tokio::task::spawn_blocking(move || store.create_own_runner(workspace_id, &saved))
            .await
            .internal()?
            .internal()?;
        let registration = self
            .registrations
            .create(target, &format!("ciproxy-{id}"))
            .await
            .internal()?;
        self.save_state(
            workspace_id,
            &runner,
            RunnerState::Starting,
            Some(registration.runner_id),
        )
        .await?;
        let message = ControlMessage::StartRunner {
            command_id: CommandId(command_id.to_string()),
            runner: RunnerSpec {
                slot_indices,
                runner_id: RunnerId(id.to_string()),
                organization_id: OrganizationId(github_organization_id.to_string()),
                label: start.label,
                size: start.requested_size,
                registration: Secret::new(registration.configuration.expose_secret()),
            },
        };
        let gateway = self.gateway.clone();
        let delivery =
            tokio::task::spawn_blocking(move || gateway.send(workspace_id, machine_id, message))
                .await
                .internal()?
                .internal()?;
        if delivery == Delivery::Offline {
            tracing::info!(%id, "runner delivery interrupted; reconciliation will retire its registration");
        }
        Ok(())
    }

    pub(crate) async fn stop(&self, workspace_id: Uuid, runner: &OwnRunner) -> Result<()> {
        let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
            return Ok(());
        };
        if runner.runner.state == RunnerState::Busy {
            return Ok(());
        }
        let mut command = runner.command_id;
        if runner.runner.state != RunnerState::Stopping {
            let target = ListenerTarget {
                workspace_id,
                organization_id: runner.runner.organization_id,
                scale_set_id: runner.runner.scale_set_id,
            };
            let github_id = match runner.runner.github_id {
                Some(id) => Some(id),
                None => self
                    .registrations
                    .find(target, &format!("ciproxy-{}", runner.runner.id))
                    .await
                    .internal()?,
            };
            if let Some(github_id) = github_id {
                self.save_state(workspace_id, runner, runner.runner.state, Some(github_id))
                    .await?;
                match self
                    .registrations
                    .remove(target, github_id)
                    .await
                    .internal()?
                {
                    Removal::Busy => {
                        self.save_state(workspace_id, runner, RunnerState::Busy, Some(github_id))
                            .await?;
                        return Ok(());
                    }
                    Removal::AccessGone => {}
                    Removal::Removed => {}
                }
            }
            command = self.clock.uuid();
            let store = self.store.clone();
            let id = runner.runner.id;
            let at = self.clock.now();
            tokio::task::spawn_blocking(move || {
                store.retire_own_runner(workspace_id, id, command, at)
            })
            .await
            .internal()?
            .internal()?;
        }
        let gateway = self.gateway.clone();
        let store = self.store.clone();
        let org_id = runner.runner.organization_id;
        let organization =
            tokio::task::spawn_blocking(move || store.organization(workspace_id, org_id))
                .await
                .internal()?
                .internal()?
                .ok_or(Error::StateChanged)?;
        let organization_id = OrganizationId(organization.github_id.to_string());
        let runner_id = RunnerId(runner.runner.id.to_string());
        let delivery = tokio::task::spawn_blocking(move || {
            gateway.send(
                workspace_id,
                machine_id,
                ControlMessage::StopRunner {
                    command_id: CommandId(command.to_string()),
                    organization_id,
                    runner_id,
                },
            )
        })
        .await
        .internal()?;
        match delivery {
            Ok(_)
            | Err(ciproxy_gateway::error::Error::NotFound)
            | Err(ciproxy_gateway::error::Error::Unauthorized) => {}
            Err(error) => return Err(error).internal(),
        }
        Ok(())
    }

    pub(crate) async fn save_state(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
        state: RunnerState,
        github_id: Option<i64>,
    ) -> Result<()> {
        let store = self.store.clone();
        let organization_id = runner.runner.organization_id;
        let id = runner.runner.id;
        tokio::task::spawn_blocking(move || {
            store.update_runner(workspace_id, organization_id, id, state, github_id)
        })
        .await
        .internal()?
        .internal()
    }
}
