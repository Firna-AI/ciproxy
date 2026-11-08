//! Existing undersized Boat runners must obey the current fallback policy.

use std::time::Duration;

use ciproxy_protocol::identifiers::{ImageId, JobId, RunnerId};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::resources::{Slots, VmSize};

use crate::actions::{Action, Destination, StartReason, StartRunner, StopReason};
use crate::boat::{BoatRunner, BoatSize, BoatSizeId, Refusal, Sandbox, SandboxId};
use crate::decision::decide;
use crate::snapshot::{NothingFits, RunnerState, Snapshot};

use super::fixtures::{
    allocated, boat, boat_size, large, own, queued, runner, sandbox, small, snapshot, warm,
};

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
    idle.size = allocated(large());
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
    state.root_sizing.image_root_bytes = 90_000_000_000;
    state.queue.push(queued("a", "large", 0));
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

#[test]
fn image_floor_blocks_largest_boat_fallback_and_retires_its_idle_runner() {
    let mut state = fallback_snapshot();
    let largest = state.boat.sizes.last_mut().unwrap();
    largest.size.disk_bytes = 60_263_759_871;
    assert_eq!(decide(&state), vec![retire_fallback()]);
    state.boat.sizes.last_mut().unwrap().size.disk_bytes = 60_263_759_872;
    let actions = decide(&state);
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0], retire_fallback());
    assert_eq!(
        actions[1],
        Action::StartRunner(StartRunner {
            target: state.labels[1].targets[0].clone(),
            organization_id: state.organizations[0].clone(),
            label: state.labels[1].name.clone(),
            requested_size: state.labels[1].size,
            size: VmSize {
                disk_bytes: 60_263_759_872,
                ..large()
            },
            destination: Destination::Boat(BoatSizeId("large".into())),
            reason: StartReason::LargestBoatSize,
        })
    );
}

#[test]
fn an_idle_boat_runner_keeps_its_saved_root_while_new_starts_use_the_current_image() {
    let mut state = snapshot();
    state.machines.clear();
    state.root_sizing.image_root_bytes = 63_000_000_000;
    let requested = VmSize {
        disk_bytes: 47_000_000_000,
        ..small()
    };
    state.labels[0].size = requested;
    state.queue = vec![queued("a", "ciproxy", 0); 2];
    let actual = VmSize {
        vcpu: 8,
        memory_bytes: 16_000_000_000,
        disk_bytes: 111_000_000_000,
    };
    state.boat.sizes = vec![BoatSize {
        id: BoatSizeId("largest".into()),
        size: actual,
        price: 2,
    }];
    state.boat.largest_size = Some(BoatSizeId("largest".into()));
    let mut retained = runner("older", "a", "ciproxy", RunnerState::Ready);
    retained.requested_size = requested;
    retained.size = actual;
    state.sandboxes = vec![Sandbox {
        id: SandboxId("older".into()),
        marked: true,
        runner: Some(BoatRunner {
            runner: retained,
            required_root_disk_bytes: 109_470_908_416,
        }),
    }];
    assert_eq!(decide(&state), vec![]);
    let fitting = VmSize {
        disk_bytes: 112_000_000_000,
        ..actual
    };
    state.boat.sizes.push(BoatSize {
        id: BoatSizeId("fitting".into()),
        size: fitting,
        price: 3,
    });
    state.boat.largest_size = Some(BoatSizeId("fitting".into()));
    assert_eq!(
        decide(&state),
        vec![Action::StartRunner(StartRunner {
            target: state.labels[0].targets[0].clone(),
            organization_id: state.organizations[0].clone(),
            label: state.labels[0].name.clone(),
            requested_size: requested,
            size: fitting,
            destination: Destination::Boat(BoatSizeId("fitting".into())),
            reason: StartReason::NoEligibleMachine,
        })]
    );
}

#[test]
fn image_growth_keeps_correctly_sized_idle_and_running_roots() {
    let mut state = snapshot();
    state.boat.enabled = false;
    state.queue = vec![queued("a", "ciproxy", 0)];
    let idle = warm("idle", "a");
    let mut running = idle.clone();
    running.id = RunnerId("running".into());
    running.state = RunnerState::Running(JobId("job".into()));
    state.machines[0].runners = vec![idle, running];
    state.root_sizing.image_root_bytes = 63_000_000_000;
    state.machines[0].selected_image = Some(ImageMetadata {
        image_id: ImageId("newer".into()),
        root_disk_bytes: 63_000_000_000,
    });
    assert_eq!(decide(&state), vec![]);
}
