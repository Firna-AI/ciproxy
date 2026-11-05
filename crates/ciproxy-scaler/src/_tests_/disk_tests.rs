//! Hand-worked job-disk pressure from the protocol's mini-PC examples.

use std::time::Duration;

use ciproxy_protocol::identifiers::{MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::decision::decide;
use crate::snapshot::RunnerState;

use super::fixtures::{boat, label, large, own, queued, running, small, snapshot, start, warm};

#[test]
fn reserves_disk_for_all_starts_in_the_same_tick() {
    let largest = VmSize {
        vcpu: 8,
        memory_bytes: 32 * GIGABYTE,
        disk_bytes: 120 * GIGABYTE,
    };
    for (name, size, disk_gb, jobs, expected) in [
        (
            "ciproxy",
            small(),
            159,
            2,
            vec![own("a", "ciproxy", small())],
        ),
        (
            "ciproxy",
            small(),
            279,
            4,
            vec![
                own("a", "ciproxy", small()),
                own("a", "ciproxy", small()),
                own("a", "ciproxy", small()),
            ],
        ),
        (
            "ciproxy-4vcpu",
            large(),
            279,
            4,
            vec![
                own("a", "ciproxy-4vcpu", large()),
                own("a", "ciproxy-4vcpu", large()),
                own("a", "ciproxy-4vcpu", large()),
            ],
        ),
        (
            "ciproxy-8vcpu",
            largest,
            279,
            2,
            vec![own("a", "ciproxy-8vcpu", largest)],
        ),
    ] {
        let mut state = snapshot();
        state.labels = vec![label(name, size)];
        state.queue = vec![queued("a", name, 0); jobs];
        state.machines[0].capacity = Slots(6);
        state.machines[0].disk_budget_bytes = disk_gb * GIGABYTE;
        state.boat.enabled = false;
        assert_eq!(decide(&state), expected, "{name}, {disk_gb} GB job disk");
    }
}

#[test]
fn warm_preemption_frees_job_disk_even_when_slots_are_available() {
    let mut state = snapshot();
    let mut size = large();
    size.disk_bytes = 120 * GIGABYTE;
    state.labels[1].size = size;
    state.machines[0].disk_budget_bytes = 279 * GIGABYTE;
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![warm("first", "a"), warm("second", "b")];
    state.queue = vec![queued("a", "large", 0)];
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("first".into()),
            reason: StopReason::WarmPreemption,
        }]
    );
    state.machines[0].runners[0].state = RunnerState::Retiring;
    assert_eq!(decide(&state), vec![]);
    state.machines[0].runners.remove(0);
    assert_eq!(decide(&state), vec![own("a", "large", size)]);
}

#[test]
fn warm_replenishment_reserves_the_job_disk_held_by_running_vms() {
    let mut state = snapshot();
    state.machines[0].disk_budget_bytes = 279 * GIGABYTE;
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![running("one"), running("two")];
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            Destination::Machine(MachineId("machine".into())),
            StartReason::WarmPool
        )]
    );
}

#[test]
fn a_label_edit_keeps_the_running_vms_original_job_disk() {
    let mut state = snapshot();
    state.machines[0].disk_budget_bytes = 279 * GIGABYTE;
    let mut old = running("old-size");
    old.size.disk_bytes = 120 * GIGABYTE;
    old.requested_size = old.size;
    state.machines[0].runners = vec![old];
    state.queue = vec![queued("a", "ciproxy", 0), queued("a", "ciproxy", 0)];
    state.boat.enabled = false;
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
}

#[test]
fn disk_pressure_keeps_the_wait_when_the_total_job_disk_fits() {
    let mut state = snapshot();
    state.machines[0].disk_budget_bytes = 279 * GIGABYTE;
    state.machines[0].runners = vec![running("one"), running("two"), running("three")];
    state.queue = vec![queued("a", "ciproxy", 29)];
    assert_eq!(decide(&state), vec![]);
    state.queue = vec![queued("a", "ciproxy", 30)];
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "ciproxy",
            small(),
            "small",
            StartReason::OverflowWaitElapsed {
                waited: Duration::from_secs(30)
            }
        )]
    );
}
