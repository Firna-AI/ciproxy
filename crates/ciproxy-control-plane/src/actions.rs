//! Persist resource reservations before sending a one-use JIT to its machine.

use ciproxy_gateway::types::Delivery;
use ciproxy_github::registrations::RegistrationRequest;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_protocol::runners::RunnerSpec;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Action, Destination, StartRunner};
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation, RoutingSnapshot};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::slots::allocate_slots;

/// State writes that supply no new execution evidence.
pub(crate) enum UnobservedRunnerUpdate {
    /// GitHub registration alone does not prove that the VM exists.
    Registered {
        github_id: i64,
        target: RegistrationTarget,
    },
    /// Absence or registration cleanup can end a reservation without new evidence.
    Stopped { github_id: Option<i64> },
}

impl MachineRuntime {
    pub(crate) async fn apply(&self, input: &RoutingSnapshot, action: Action) -> Result<()> {
        match action {
            Action::StartRunner(start) => {
                if matches!(start.destination, Destination::Boat(_)) {
                    self.start_boat(input, start).await
                } else {
                    self.start(input, start).await
                }
            }
            Action::StopRunner { runner_id, .. } => {
                let runner = input
                    .runners
                    .iter()
                    .find(|runner| runner.runner.id.to_string() == runner_id.0);
                if let Some(runner) = runner {
                    self.stop(input, runner).await
                } else {
                    self.cleanup_boat(input.workspace_id).await
                }
            }
            Action::DeleteSandbox { .. } => self.cleanup_boat(input.workspace_id).await,
        }
    }

    async fn start(&self, input: &RoutingSnapshot, start: StartRunner) -> Result<()> {
        let Destination::Machine(machine_id) = start.destination else {
            return Ok(());
        };
        let machine_id: Uuid = machine_id.0.parse().internal()?;
        let organization_id: Uuid = start.target.organization_id.0.parse().internal()?;
        let github_organization_id = input
            .organizations
            .iter()
            .find(|org| org.id == organization_id)
            .ok_or(Error::StateChanged)?
            .github_id;
        let label = input
            .labels
            .iter()
            .find(|label| label.id.to_string() == start.target.label_id)
            .ok_or(Error::StateChanged)?;
        let workspace_id = input.workspace_id;
        let set = input
            .scale_sets
            .iter()
            .find(|set| {
                set.id.to_string() == start.target.scale_set_id
                    && set.organization_id == organization_id
                    && set.label_id == label.id
                    && set.github_id == start.target.github_id
            })
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
            .map_or(0, |report| report.report.capacity.0);
        let slot_indices = allocate_slots(machine_id, start.size, capacity, &reservations)
            .ok_or(Error::StateChanged)?;
        let id = self.clock.uuid();
        let command_id = self.clock.uuid();
        let request = RegistrationRequest {
            workspace_id,
            target: RegistrationTarget::from(set),
        };
        let runner = OwnRunner {
            registration_scale_set_id: None,
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
            confirmed_at: None,
        };
        let store = self.store.clone();
        let saved = runner.clone();
        tokio::task::spawn_blocking(move || store.create_own_runner(workspace_id, &saved))
            .await
            .internal()?
            .internal()?;
        let registration = self
            .registrations
            .create(request, &format!("ciproxy-{id}"))
            .await
            .internal()?;
        self.save_state(
            workspace_id,
            &runner,
            UnobservedRunnerUpdate::Registered {
                github_id: registration.runner_id,
                target: registration.target,
            },
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

    pub(crate) async fn save_state(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
        update: UnobservedRunnerUpdate,
    ) -> Result<()> {
        let (state, github_id) = match update {
            UnobservedRunnerUpdate::Registered { github_id, target } => {
                let store = self.store.clone();
                let id = runner.runner.id;
                return tokio::task::spawn_blocking(move || {
                    store.save_runner_registration(workspace_id, id, target, github_id)
                })
                .await
                .internal()?
                .internal();
            }
            UnobservedRunnerUpdate::Stopped { github_id } => (RunnerState::Stopped, github_id),
        };
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

    pub(crate) async fn observe_runner(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
        observation: OwnRunnerObservation,
    ) -> Result<()> {
        let store = self.store.clone();
        let organization_id = runner.runner.organization_id;
        let id = runner.runner.id;
        tokio::task::spawn_blocking(move || {
            store.observe_own_runner(workspace_id, organization_id, id, observation)
        })
        .await
        .internal()?
        .internal()
    }
}
