//! Destination edits retire idle supply before replacing it elsewhere.

use ciproxy_protocol::identifiers::RunnerId;

use crate::actions::{Action, StartReason, StopReason};
use crate::decision::decide;
use crate::provider::SandboxId;
use crate::snapshot::{Place, RunnerState};

use super::fixtures::{BOAT, boat, own, queued, runner, running, sandbox, small, snapshot};

#[test]
fn obeys_orders_with_one_or_no_place() {
    for (own_enabled, boat_enabled, expected) in [
        (true, false, vec![own("a", "ciproxy", small())]),
        (
            false,
            true,
            vec![boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::NoEligibleMachine,
            )],
        ),
        (false, false, vec![]),
    ] {
        let mut state = snapshot();
        state.queue = vec![queued("a", "ciproxy", 100)];
        state.labels[0].order = [
            own_enabled.then_some(Place::OwnMachines),
            boat_enabled.then_some(Place::Provider(BOAT)),
        ]
        .into_iter()
        .flatten()
        .collect();
        assert_eq!(
            decide(&state),
            expected,
            "own {own_enabled}, Boat {boat_enabled}"
        );
    }

    let mut boat_only = snapshot();
    boat_only.queue = vec![queued("a", "ciproxy", 0)];
    boat_only.labels[0]
        .order
        .retain(|place| *place != Place::OwnMachines);
    boat_only.machines[0].runners = vec![
        runner("idle-own", "a", "ciproxy", RunnerState::Ready),
        running("busy-own"),
    ];
    assert_eq!(
        decide(&boat_only),
        vec![
            Action::StopRunner {
                runner_id: RunnerId("idle-own".into()),
                reason: StopReason::DestinationDisabled,
            },
            boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::NoEligibleMachine
            ),
        ]
    );

    let mut own_only = snapshot();
    own_only.queue = vec![queued("a", "ciproxy", 0)];
    own_only.labels[0]
        .order
        .retain(|place| *place != Place::Provider(BOAT));
    own_only.sandboxes = vec![
        sandbox(
            "idle-boat",
            Some(runner(
                "idle-boat-runner",
                "a",
                "ciproxy",
                RunnerState::Ready,
            )),
        ),
        sandbox("busy-boat", Some(running("busy-boat-runner"))),
    ];
    assert_eq!(
        decide(&own_only),
        vec![
            Action::DeleteSandbox {
                provider: BOAT,
                sandbox_id: SandboxId("idle-boat".into()),
                reason: StopReason::DestinationDisabled,
            },
            own("a", "ciproxy", small()),
        ]
    );
}
