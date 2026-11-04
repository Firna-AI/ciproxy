//! Own-first, overflow eligibility, catalog selection, and refusal rules.

use std::time::Duration;

use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::StartReason;
use crate::boat::BoatSizeId;
use crate::decision::decide;
use crate::snapshot::NothingFits;

use super::fixtures::{
    boat, boat_size, boat_with_request, large, own, queued, running, small, snapshot,
};

#[test]
fn starts_on_free_own_capacity_before_boat() {
    let mut state = snapshot();
    state.queue = vec![queued("a", "large", 80), queued("b", "large", 80)];
    state.machines[0].capacity = Slots(3);
    assert_eq!(
        decide(&state),
        vec![
            own("a", "large", large()),
            boat(
                "b",
                "large",
                large(),
                "large",
                StartReason::OverflowWaitElapsed {
                    waited: Duration::from_secs(80)
                }
            ),
        ]
    );
}

#[test]
fn overflow_starts_at_the_wait_boundary_only_when_machines_are_busy() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(1);
    state.machines[0].runners = vec![running("busy")];
    for (age, expected) in [
        (29, vec![]),
        (
            30,
            vec![boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::OverflowWaitElapsed {
                    waited: Duration::from_secs(30),
                },
            )],
        ),
    ] {
        state.queue = vec![queued("a", "ciproxy", age)];
        assert_eq!(decide(&state), expected, "queue age {age}");
    }
    state.overflow_wait = Duration::ZERO;
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "ciproxy",
            small(),
            "small",
            StartReason::OverflowWaitElapsed {
                waited: Duration::ZERO
            }
        )]
    );
}

#[test]
fn ineligible_own_machines_do_not_add_an_overflow_wait() {
    for case in ["offline", "draining", "slots", "disk", "disallowed"] {
        let mut state = snapshot();
        state.queue = vec![queued("a", "ciproxy", 0)];
        match case {
            "offline" => state.machines[0].connected = false,
            "draining" => state.machines[0].draining = true,
            "slots" => state.machines[0].capacity = Slots(0),
            "disk" => state.machines[0].disk_budget_bytes = 59 * GIGABYTE,
            "disallowed" => state.labels[0].destinations.own_machines = false,
            _ => unreachable!(),
        }
        assert_eq!(
            decide(&state),
            vec![boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::NoEligibleMachine
            )],
            "{case}"
        );
    }
}

#[test]
fn chooses_the_cheapest_boat_size_that_covers_all_dimensions() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "large", 0)];
    state.boat.sizes = vec![
        boat_size("cpu-short", VmSize { vcpu: 3, ..large() }, 1),
        boat_size(
            "memory-short",
            VmSize {
                memory_bytes: 15 * GIGABYTE,
                ..large()
            },
            1,
        ),
        boat_size(
            "disk-short",
            VmSize {
                disk_bytes: 79 * GIGABYTE,
                ..large()
            },
            1,
        ),
        boat_size("expensive", large(), 9),
        boat_size("best", large(), 3),
    ];
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "large",
            large(),
            "best",
            StartReason::NoEligibleMachine
        )]
    );
}

#[test]
fn nothing_fits_waits_or_uses_the_explicit_largest_size() {
    let mut state = snapshot();
    state.machines.clear();
    state.labels[1].size = VmSize {
        vcpu: 16,
        ..large()
    };
    state.queue = vec![queued("a", "large", 100)];
    assert_eq!(decide(&state), vec![]);
    state.labels[1].nothing_fits = NothingFits::LargestBoatSize;
    assert_eq!(
        decide(&state),
        vec![boat_with_request(
            "a",
            "large",
            VmSize {
                vcpu: 16,
                ..large()
            },
            large(),
            "large",
            StartReason::LargestBoatSize
        )]
    );
    state.labels[1].destinations.boat = false;
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn never_uses_a_smaller_boat_vm_while_an_eligible_own_machine_fits() {
    let mut state = snapshot();
    state.boat.sizes.truncate(1);
    state.boat.largest_size = Some(BoatSizeId("small".into()));
    state.labels[1].nothing_fits = NothingFits::LargestBoatSize;
    state.machines[0].capacity = Slots(2);
    let mut occupied = running("busy");
    occupied.size = large();
    state.machines[0].runners = vec![occupied];
    state.queue = vec![queued("a", "large", 100)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn free_slots_on_separate_machines_cannot_form_one_job_vm() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(2);
    state.machines[0].runners = vec![running("busy-one")];
    let mut second = state.machines[0].clone();
    second.id.0 = "second".into();
    second.runners = vec![running("busy-two")];
    state.machines.push(second);
    state.queue = vec![queued("a", "large", 0)];
    assert_eq!(decide(&state), vec![]);
    state.queue = vec![queued("a", "large", 30)];
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "large",
            large(),
            "large",
            StartReason::OverflowWaitElapsed {
                waited: Duration::from_secs(30)
            }
        )]
    );
}
