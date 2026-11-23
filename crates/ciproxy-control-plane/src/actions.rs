//! Persist resource reservations before sending a one-use JIT to its machine.

use ciproxy_scaler::actions::{Action, Destination};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation, RoutingSnapshot};
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::snapshot::provider_id;

/// State writes that supply no new execution evidence.
pub(crate) enum UnobservedRunnerUpdate {
    /// Absence or registration cleanup can end a reservation without new evidence.
    Stopped { github_id: Option<i64> },
}

impl MachineRuntime {
    pub(crate) async fn apply(&self, input: &RoutingSnapshot, action: Action) -> Result<()> {
        let boat = provider_id(OverflowProvider::Boat);
        match action {
            Action::StartRunner(start) => match start.destination {
                Destination::Machine(_) => self.start(input, *start).await,
                Destination::Provider { provider, .. } if provider == boat => {
                    self.start_boat(input, *start).await
                }
                Destination::Provider { .. } => Err(Error::StateChanged),
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
                    self.cleanup_boat(input.workspace_id).await
                }
            }
            Action::DeleteSandbox { provider, .. } if provider == boat => {
                self.cleanup_boat(input.workspace_id).await
            }
            Action::DeleteSandbox { .. } => Err(Error::StateChanged),
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
