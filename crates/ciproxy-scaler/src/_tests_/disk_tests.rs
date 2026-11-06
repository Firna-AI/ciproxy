//! Hand-worked job-disk pressure from the protocol's mini-PC examples.

use std::time::Duration;

use ciproxy_protocol::identifiers::{ImageId, MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::{Action, Destination, StartReason, StartRunner, StopReason};
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
    old.size.disk_bytes = 122_136_727_552;
    old.requested_size.disk_bytes = 60 * GIGABYTE;
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

#[test]
fn mixed_image_releases_use_each_machines_root_for_starts_and_warm_capacity() {
    let mut state = snapshot();
    state.root_sizing.image_root_bytes = 70_000_000_000;
    state.boat.enabled = false;
    state.machines[0].disk_budget_bytes = 82_500_000_000;
    state.machines[0].selected_image = Some(ImageMetadata {
        image_id: ImageId("newer".into()),
        root_disk_bytes: 63_000_000_000,
    });
    let mut older = state.machines[0].clone();
    older.id = MachineId("older".into());
    older.selected_image = Some(ImageMetadata {
        image_id: ImageId("older".into()),
        root_disk_bytes: 61_000_000_000,
    });
    state.machines.push(older);
    state.queue = vec![queued("a", "ciproxy", 0); 2];
    let expected = StartRunner {
        organization_id: OrganizationId("a".into()),
        label: state.labels[0].name.clone(),
        requested_size: small(),
        size: VmSize {
            disk_bytes: 81_635_856_384,
            ..small()
        },
        destination: Destination::Machine(MachineId("older".into())),
        reason: StartReason::OwnMachineCapacity,
    };
    assert_eq!(decide(&state), vec![Action::StartRunner(expected.clone())]);
    state.queue.clear();
    for machine in &mut state.machines {
        machine.warm_count = 2;
        machine.warm_allocation = vec![OrganizationId("a".into()); 2];
    }
    assert_eq!(
        decide(&state),
        vec![Action::StartRunner(StartRunner {
            reason: StartReason::WarmPool,
            ..expected
        })]
    );
}
