//! Persist registration retirement after a Boat delivery can no longer launch.

use ciproxy_github::persistence::lease::Lease;
use ciproxy_overflow::api::Retirement;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runtime_results::BoatRunnerResult;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::retirement::RunnerRetirement;
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    /// Collect current Busy or durable retirement for every scoped Boat intent.
    pub(crate) async fn retire_boat_registrations(
        &self,
        input: &RoutingSnapshot,
        organization_id: Uuid,
        lease: &dyn Lease,
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
            lease.check().await.internal()?;
            if self
                .current_boat_busy(input.workspace_id, intent.runner_id)
                .await?
            {
                results.push(RunnerRetirement::Busy);
                continue;
            }
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
        let store = self.store.clone();
        let runner = intent.runner_id;
        let access_ended = tokio::task::spawn_blocking(move || {
            store.boat_registration_access_ended(workspace, runner)
        })
        .await
        .internal()?
        .internal()?;
        if access_ended {
            return Ok(RunnerRetirement::Deferred);
        }
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
        if self.task_scope()?.contains(Work::Create(intent.runner_id)) {
            return Ok(RunnerRetirement::Pending);
        }
        let store = self.store.clone();
        let runner = intent.runner_id;
        let Some(expected) =
            tokio::task::spawn_blocking(move || store.boat_runner_guard(workspace, runner))
                .await
                .internal()?
                .internal()?
        else {
            return Ok(RunnerRetirement::Retired);
        };
        let authority = authority.clone();
        let intent = intent.clone();
        let generation = self
            .takeout_checks
            .lock()
            .await
            .get(&intent.organization_id)
            .map(|check| check.generation);
        let runtime = self.worker();
        self.dispatch(Work::BoatRetire(runner), async move {
            let answer = match authority.remove(workspace, &intent).await {
                Ok(answer) => answer,
                Err(error) => {
                    tracing::warn!(%error,"Boat registration retirement remains pending");
                    if let Some(check) = runtime
                        .takeout_checks
                        .lock()
                        .await
                        .get_mut(&intent.organization_id)
                        && Some(check.generation) == generation
                    {
                        check.failed = true;
                    }
                    return Err(Error::StateChanged);
                }
            };
            match answer.retirement {
                Retirement::Removed if withdrawn => {
                    let store = runtime.store.clone();
                    tokio::task::spawn_blocking(move || {
                        store.apply_boat_runner_result(
                            workspace,
                            &expected,
                            BoatRunnerResult::Retired,
                        )
                    })
                    .await
                    .internal()?
                    .internal()?;
                }
                Retirement::RunningJob if running => {
                    if let Some(generation) = generation {
                        let current = runtime
                            .takeout_checks
                            .lock()
                            .await
                            .get(&intent.organization_id)
                            .map(|check| check.generation);
                        if current == Some(generation) {
                            runtime
                                .boat_answers
                                .lock()
                                .await
                                .insert(runner, (generation, answer.guard));
                        }
                    }
                }
                _ => (),
            }
            Ok(())
        })?;
        Ok(RunnerRetirement::Pending)
    }
}
