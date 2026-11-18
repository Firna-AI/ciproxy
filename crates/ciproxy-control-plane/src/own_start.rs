//! Persist resource reservations before sending a one-use JIT to its machine.

use ciproxy_gateway::types::Delivery;
use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_github::registrations::RegistrationRequest;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_protocol::runners::RunnerSpec;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartRunner};
use ciproxy_store_interface::own_runners::{OwnRunner, RoutingSnapshot};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::runtime_results::{OwnRunnerGuard, OwnRunnerResult};
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;
use crate::slots::allocate_slots;

struct OwnStart {
    workspace_id: Uuid,
    machine_id: Uuid,
    id: Uuid,
    command_id: Uuid,
    github_organization_id: i64,
    slot_indices: Vec<u64>,
    request: RegistrationRequest,
    start: StartRunner,
    guard: OwnRunnerGuard,
}

impl MachineRuntime {
    pub(crate) async fn start(&self, input: &RoutingSnapshot, start: StartRunner) -> Result<()> {
        let Destination::Machine(machine_id) = &start.destination else {
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
            worker_job_id: None,
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
        let guard = self.runner_guard(workspace_id, &runner).await?;
        let runtime = self.worker();
        self.dispatch(Work::Create(id), async move {
            runtime
                .execute_own_start(OwnStart {
                    workspace_id,
                    machine_id,
                    id,
                    command_id,
                    github_organization_id,
                    slot_indices,
                    request,
                    start,
                    guard,
                })
                .await
        })
    }

    async fn execute_own_start(&self, pending: OwnStart) -> Result<()> {
        let OwnStart {
            workspace_id,
            machine_id,
            id,
            command_id,
            github_organization_id,
            slot_indices,
            request,
            start,
            guard,
        } = pending;

        let registration = self
            .registrations
            .create(request, &format!("ciproxy-{id}"))
            .await
            .internal()?;
        let target = registration.target;
        let github_id = registration.runner_id;
        let accepted = self
            .apply_remote(
                workspace_id,
                guard.clone(),
                OwnRunnerResult::Registered { github_id, target },
            )
            .await?;
        if !accepted {
            return self.cleanup_late_registration(workspace_id, id, target, github_id);
        }
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
        let _operation = self.operation.lock().await;
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return self.cleanup_late_registration(workspace_id, id, target, github_id);
        };
        lease.check().await.internal()?;
        let mut expected = guard;
        expected.runner.runner.github_id = Some(github_id);
        expected.runner.registration_scale_set_id = Some(target.github_id);
        let store = self.store.clone();
        let current = tokio::task::spawn_blocking(move || {
            store.apply_own_runner_result(workspace_id, &expected, OwnRunnerResult::Prepared)
        })
        .await
        .internal()?
        .internal()?;
        if !current {
            return self.cleanup_late_registration(workspace_id, id, target, github_id);
        }
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
}
