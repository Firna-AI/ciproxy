//! Boat runner readiness expires only for launched idle runners.

use std::time::Duration;

use crate::actions::{Action, StartReason, StopReason};
use crate::decision::decide;
use crate::provider::SandboxId;
use crate::snapshot::RunnerState;

use super::fixtures::{BOAT, boat, queued, runner, running, sandbox, small, snapshot};

#[test]
fn replaces_a_launched_idle_boat_runner_at_but_not_before_the_ready_deadline() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 0)];
    let mut retained = sandbox(
        "idle",
        Some(runner("boat", "a", "ciproxy", RunnerState::Ready)),
    );
    retained.runner.as_mut().unwrap().launched_at = Some(Duration::from_secs(100));
    state.sandboxes = vec![retained];
    state.now = Duration::from_secs(699);
    assert_eq!(decide(&state), vec![], "one second before the deadline");

    state.now = Duration::from_secs(700);
    assert_eq!(
        decide(&state),
        vec![
            Action::DeleteSandbox {
                provider: BOAT,
                sandbox_id: SandboxId("idle".into()),
                reason: StopReason::ReadyDeadlinePassed,
            },
            boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::NoEligibleMachine
            ),
        ],
        "at the deadline"
    );
}

#[test]
fn keeps_a_launched_boat_runner_with_a_job_past_the_ready_deadline() {
    let mut state = snapshot();
    state.machines.clear();
    let mut retained = sandbox("busy", Some(running("boat")));
    retained.runner.as_mut().unwrap().launched_at = Some(Duration::from_secs(100));
    state.sandboxes = vec![retained];
    state.now = Duration::from_secs(701);

    assert_eq!(decide(&state), vec![]);
}

#[test]
fn a_boat_runner_without_a_launch_time_still_covers_waiting_demand() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 0)];
    state.sandboxes = vec![sandbox(
        "older",
        Some(runner("boat", "a", "ciproxy", RunnerState::Ready)),
    )];
    state.now = Duration::from_secs(3600);

    assert_eq!(decide(&state), vec![]);
}
