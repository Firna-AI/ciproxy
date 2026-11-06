//! Ready capacity covers demand before any starting capacity at any destination.

use ciproxy_protocol::identifiers::RunnerId;

use crate::actions::{Action, StopReason};
use crate::boat::SandboxId;
use crate::decision::decide;
use crate::snapshot::RunnerState;

use super::fixtures::{allocated, large, queued, runner, sandbox, snapshot};

#[test]
fn ready_boat_supply_is_kept_before_starting_supply_in_either_order() {
    for ready_first in [true, false] {
        let mut state = snapshot();
        state.machines.clear();
        state.queue = vec![queued("a", "ciproxy", 0)];
        let ready = sandbox(
            "ready",
            Some(runner("ready-runner", "a", "ciproxy", RunnerState::Ready)),
        );
        let starting = sandbox(
            "starting",
            Some(runner(
                "starting-runner",
                "a",
                "ciproxy",
                RunnerState::Starting,
            )),
        );
        state.sandboxes = if ready_first {
            vec![ready, starting]
        } else {
            vec![starting, ready]
        };
        assert_eq!(
            decide(&state),
            vec![Action::DeleteSandbox {
                sandbox_id: SandboxId("starting".into()),
                reason: StopReason::NoWaitingJob,
            }],
            "ready-first order: {ready_first}"
        );
    }
}

#[test]
fn ready_boat_supply_outranks_a_starting_own_runner() {
    let mut state = snapshot();
    state.queue = vec![queued("a", "large", 0)];
    let mut starting = runner("own-starting", "a", "large", RunnerState::Starting);
    starting.requested_size = large();
    starting.size = allocated(large());
    let mut ready = starting.clone();
    ready.id = RunnerId("boat-ready".into());
    ready.state = RunnerState::Ready;
    state.machines[0].runners = vec![starting];
    state.sandboxes = vec![sandbox("ready", Some(ready))];
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("own-starting".into()),
            reason: StopReason::NoWaitingJob,
        }]
    );
}
