//! One saved retirement answer for every machine or Boat reservation.

use std::collections::BTreeSet;

use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use uuid::Uuid;

use crate::retirement::RunnerRetirement;

/// Include registration-only, delivery-only, and materialized runner reservations.
pub(crate) fn retirement_results(
    input: &RoutingSnapshot,
    organization_id: Uuid,
    runners: &[Runner],
) -> Vec<RunnerRetirement> {
    let mut results: Vec<_> = input
        .runners
        .iter()
        .filter(|runner| runner.runner.organization_id == organization_id)
        .map(|runner| match runner.runner.state {
            RunnerState::Busy => RunnerRetirement::Busy,
            RunnerState::Stopping | RunnerState::Stopped => RunnerRetirement::Retired,
            _ => RunnerRetirement::Pending,
        })
        .collect();
    let intents = input
        .boat_intents
        .iter()
        .filter(|intent| intent.organization_id == organization_id);
    let deliveries = input
        .overflow
        .starts
        .iter()
        .filter(|record| record.organization_id == organization_id);
    let boat_runners = runners
        .iter()
        .filter(|runner| matches!(runner.place, RunnerPlace::Boat { .. }));
    let runner_ids: BTreeSet<_> = intents
        .map(|intent| intent.runner_id)
        .chain(deliveries.map(|record| record.runner_id))
        .chain(boat_runners.map(|runner| runner.id))
        .collect();
    for runner_id in runner_ids {
        results.push(boat_result(
            input
                .boat_intents
                .iter()
                .find(|intent| intent.runner_id == runner_id),
            input
                .overflow
                .starts
                .iter()
                .find(|record| record.runner_id == runner_id),
            runners.iter().find(|runner| runner.id == runner_id),
        ));
    }
    results
}

fn boat_result(
    intent: Option<&BoatRunnerIntent>,
    delivery: Option<&StartRecord>,
    runner: Option<&Runner>,
) -> RunnerRetirement {
    match delivery {
        Some(record) if record.ends_at.is_some() || record.phase == StartPhase::Deleting => {
            if intent.is_none() {
                RunnerRetirement::Retired
            } else {
                RunnerRetirement::Pending
            }
        }
        Some(record)
            if record.phase == StartPhase::Running
                && intent.is_some()
                && runner.is_some_and(|runner| runner.state == RunnerState::Busy) =>
        {
            RunnerRetirement::Busy
        }
        None if intent.is_none()
            && runner.is_some_and(|runner| runner.state == RunnerState::Stopped) =>
        {
            RunnerRetirement::Retired
        }
        _ => RunnerRetirement::Pending,
    }
}
