//! Reconcile exact resources and slot ownership from authenticated full reports.

use ciproxy_protocol::runners;
use ciproxy_store_interface::own_runners::{OwnRunner, RoutingSnapshot};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::vm_size::VmSize;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn reconcile_reports(&self, input: &RoutingSnapshot) -> Result<()> {
        for runner in &input.runners {
            match self.reconcile_report(input, runner).await {
                Ok(()) => {}
                Err(Error::StateChanged) => return Err(Error::StateChanged),
                Err(error) => {
                    tracing::warn!(runner=%runner.runner.id,%error,"runner reconciliation will retry; independent runners continue")
                }
            }
        }
        Ok(())
    }

    async fn reconcile_report(&self, input: &RoutingSnapshot, runner: &OwnRunner) -> Result<()> {
        if runner.runner.state == RunnerState::Stopped {
            return Ok(());
        }
        let RunnerPlace::Machine { machine_id, .. } = runner.runner.place else {
            return Ok(());
        };
        let Some(machine) = input.machines.iter().find(|m| m.machine.id == machine_id) else {
            self.stop(input.workspace_id, runner).await?;
            return Ok(());
        };
        let Some(report) = &machine.report else {
            return Ok(());
        };
        if let Some(observed) = report
            .runners
            .iter()
            .find(|report| report.runner_id.0 == runner.runner.id.to_string())
        {
            let requested = match VmSize::from_request(observed.requested_size) {
                Ok(requested) => requested,
                Err(error) => {
                    tracing::warn!(runner=%runner.runner.id,%error,"unknown request units; preserve original label request");
                    runner.requested_size
                }
            };
            if observed.size != runner.runner.size || requested != runner.requested_size {
                let store = self.store.clone();
                let workspace_id = input.workspace_id;
                let runner_id = runner.runner.id;
                let allocation = observed.size;
                tokio::task::spawn_blocking(move || {
                    store.save_own_size(workspace_id, runner_id, requested, allocation)
                })
                .await
                .internal()?
                .internal()?;
            }
            if observed.slot_indices != runner.slot_indices {
                let store = self.store.clone();
                let workspace_id = input.workspace_id;
                let runner_id = runner.runner.id;
                let indices = observed.slot_indices.clone();
                tokio::task::spawn_blocking(move || {
                    store.save_own_slots(workspace_id, runner_id, &indices)
                })
                .await
                .internal()?
                .internal()?;
            }
            let state = match observed.state {
                runners::RunnerState::Starting => RunnerState::Starting,
                runners::RunnerState::Warm => RunnerState::Ready,
                runners::RunnerState::Running { .. } => RunnerState::Busy,
                runners::RunnerState::Stopping => RunnerState::Stopping,
            };
            if runner.runner.state != RunnerState::Stopping
                && runner.runner.state != RunnerState::Busy
                && runner.runner.state != state
            {
                self.save_state(input.workspace_id, runner, state, runner.runner.github_id)
                    .await?;
            }
            if runner.runner.state == RunnerState::Stopping && state == RunnerState::Busy {
                self.save_state(
                    input.workspace_id,
                    runner,
                    RunnerState::Busy,
                    runner.runner.github_id,
                )
                .await?;
            }
        } else if machine
            .machine
            .last_contact_at
            .is_some_and(|at| at > runner.issued_at)
        {
            if matches!(
                runner.runner.state,
                RunnerState::Stopping | RunnerState::Busy
            ) {
                self.save_state(
                    input.workspace_id,
                    runner,
                    RunnerState::Stopped,
                    runner.runner.github_id,
                )
                .await?;
            } else if runner.acknowledged
                || runner.issued_at + chrono::Duration::seconds(60) < self.clock.now()
            {
                self.stop(input.workspace_id, runner).await?;
            }
        }
        Ok(())
    }
}
