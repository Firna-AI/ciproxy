//! Persist resource reservations before sending a one-use JIT to its machine.

use ciproxy_scaler::actions::{Action, Destination};
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation, RoutingSnapshot};
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

/// State writes that supply no new execution evidence.
pub(crate) enum UnobservedRunnerUpdate {
    /// Absence or registration cleanup can end a reservation without new evidence.
    Stopped { github_id: Option<i64> },
}

impl MachineRuntime {
    pub(crate) async fn apply(&self, input: &RoutingSnapshot, action: Action) -> Result<()> {
        match action {
            Action::StartRunner(start) => match start.destination {
                Destination::Machine(_) => self.start(input, *start).await,
                Destination::Provider { .. } => self.start_overflow(input, *start).await,
            },
            Action::StopRunner { runner_id, reason } => {
                tracing::info!(runner = %runner_id.0, ?reason, "retire an idle runner");
                let runner = input
                    .runners
                    .iter()
                    .find(|runner| runner.runner.id.to_string() == runner_id.0);
                if let Some(runner) = runner {
                    self.stop(input, runner).await
                } else {
                    self.cleanup_overflow(input.workspace_id).await
                }
            }
            Action::DeleteSandbox { .. } => self.cleanup_overflow(input.workspace_id).await,
        }
    }

    pub(crate) async fn save_state(
        &self,
        workspace_id: Uuid,
        runner: &OwnRunner,
        update: UnobservedRunnerUpdate,
    ) -> Result<()> {
        let UnobservedRunnerUpdate::Stopped { github_id } = update;
        let state = RunnerState::Stopped;
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
