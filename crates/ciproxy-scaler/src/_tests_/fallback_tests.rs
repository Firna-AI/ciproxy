//! Existing undersized Boat runners must obey the current fallback policy.

use std::time::Duration;

use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::resources::{Slots, VmSize};

use crate::actions::{Action, StartReason, StopReason};
use crate::boat::{BoatSizeId, Refusal, SandboxId};
use crate::decision::decide;
use crate::snapshot::{NothingFits, RunnerState, Snapshot};

use super::fixtures::{boat, boat_size, large, own, queued, runner, sandbox, snapshot};

fn fallback_snapshot() -> Snapshot {
    let mut state = snapshot();
    state.machines.clear();
    state.labels[1].size = VmSize {
        vcpu: 16,
        ..large()
    };
    state.labels[1].nothing_fits = NothingFits::LargestBoatSize;
    state.queue = vec![queued("a", "large", 0)];
    let mut idle = runner("fallback-idle", "a", "large", RunnerState::Ready);
    idle.requested_size = VmSize {
        vcpu: 16,
        ..large()
    };
    idle.size = large();
    let mut busy = idle.clone();
    busy.id = RunnerId("fallback-running".into());
    busy.state = RunnerState::Running(JobId("running-job".into()));
    state.sandboxes = vec![sandbox("idle", Some(idle)), sandbox("busy", Some(busy))];
    state
}

fn retire_fallback() -> Action {
    Action::DeleteSandbox {
        sandbox_id: SandboxId("idle".into()),
        reason: StopReason::SizeNoLongerAllowed,
    }
}

#[test]
fn wait_retires_an_idle_undersized_boat_fallback() {
    let mut state = fallback_snapshot();
    assert_eq!(decide(&state), vec![]);
    state.boat.enabled = false;
    state.boat.connected = false;
    state.boat.spend.spent = 100;
    state.boat.refusals = vec![
        Refusal::RejectedKey,
        Refusal::BillingRequired,
        Refusal::RetryAfter {
            not_before: Duration::from_secs(200),
        },
    ];
    assert_eq!(decide(&state), vec![]);
    state.labels[1].nothing_fits = NothingFits::Wait;
    assert_eq!(decide(&state), vec![retire_fallback()]);
}

#[test]
fn a_fitting_machine_retires_an_idle_undersized_boat_fallback() {
    let mut state = fallback_snapshot();
    assert_eq!(decide(&state), vec![]);
    state.machines = snapshot().machines;
    state.machines[0].capacity = Slots(8);
    assert_eq!(
        decide(&state),
        vec![
            retire_fallback(),
            own(
                "a",
                "large",
                VmSize {
                    vcpu: 16,
                    ..large()
                }
            ),
        ]
    );
}

#[test]
fn a_fitting_catalog_retires_an_idle_undersized_boat_fallback() {
    let mut state = fallback_snapshot();
    assert_eq!(decide(&state), vec![]);
    state.boat.sizes.push(boat_size(
        "fitting",
        VmSize {
            vcpu: 16,
            ..large()
        },
        9,
    ));
    state.boat.largest_size = Some(BoatSizeId("fitting".into()));
    assert_eq!(
        decide(&state),
        vec![
            retire_fallback(),
            boat(
                "a",
                "large",
                VmSize {
                    vcpu: 16,
                    ..large()
                },
                "fitting",
                StartReason::NoEligibleMachine
            ),
        ]
    );
}
