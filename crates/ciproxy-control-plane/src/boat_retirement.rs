//! Persist registration retirement after a Boat delivery can no longer launch.

use ciproxy_overflow::api::Retirement;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::{OwnRunnerLease, RoutingSnapshot};
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::retirement::RunnerRetirement;
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    /// Collect current Busy or durable retirement for every scoped Boat intent.
    pub(crate) async fn retire_boat_registrations(
        &self,
        input: &RoutingSnapshot,
        organization_id: Uuid,
        lease: &mut dyn OwnRunnerLease,
    ) -> Result<Vec<RunnerRetirement>> {
        let mut results = Vec::new();
        for intent in input
            .boat_intents
            .iter()
            .filter(|intent| intent.organization_id == organization_id)
        {
            let delivery = input
                .overflow
                .starts
                .iter()
                .find(|record| record.delivery_id == intent.delivery_id);
            lease.check_connection().internal()?;
            match self
                .retire_boat_registration(input.workspace_id, intent, delivery)
                .await
            {
                Ok(result) => results.push(result),
                Err(error) => {
                    results.push(RunnerRetirement::Pending);
                    tracing::warn!(runner=%intent.runner_id,%error,"Boat registration retirement will retry");
                }
            }
        }
        Ok(results)
    }

    /// Save registration retirement or Busy proof without releasing a live delivery.
    pub(crate) async fn retire_boat_registration(
        &self,
        workspace: Uuid,
        intent: &BoatRunnerIntent,
        delivery: Option<&StartRecord>,
    ) -> Result<RunnerRetirement> {
        let withdrawn = delivery
            .is_none_or(|record| record.ends_at.is_some() || record.phase == StartPhase::Deleting);
        let running = delivery
            .is_some_and(|record| record.phase == StartPhase::Running && record.ends_at.is_none());
        if !withdrawn && !running {
            return Ok(RunnerRetirement::Pending);
        }
        let Some(authority) = &self.boat_retirement else {
            return Ok(RunnerRetirement::Pending);
        };
        match authority.remove(workspace, intent).await.internal()? {
            Retirement::Removed if withdrawn => {
                let store = self.store.clone();
                let runner = intent.runner_id;
                tokio::task::spawn_blocking(move || {
                    store.remove_boat_runner_intent(workspace, runner)
                })
                .await
                .internal()?
                .internal()?;
                Ok(RunnerRetirement::Retired)
            }
            Retirement::RunningJob if running => Ok(RunnerRetirement::Busy),
            _ => Ok(RunnerRetirement::Pending),
        }
    }
}
