//! A job runs only on a machine of its label's platform, and macOS never uses Boat.

use ciproxy_agent::root_sizing::MACOS_ROOT_MARGIN_BYTES;
use ciproxy_protocol::identifiers::{ImageId, JobId, MachineId, OrganizationId};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::GIGABYTE;

use crate::actions::{Action, Destination, StartReason};
use crate::decision::decide;
use crate::snapshot::{Machine, NothingFits, RunnerState, Snapshot};

use super::fixtures::{boat, label, queued, runner, small, snapshot, start};

fn mac(id: &str) -> Machine {
    let mut machine = snapshot().machines.remove(0);
    machine.id = MachineId(id.into());
    machine.platform = Platform::MacosArm64;
    machine.selected_image = Some(standard_macos_image());
    machine
}

fn with_mac_label() -> Snapshot {
    let mut state = snapshot();
    let mut mac_label = label("macos", small());
    mac_label.platform = Platform::MacosArm64;
    state.labels.push(mac_label);
    state
}

fn mac_start(machine: &str) -> Action {
    mac_runner_start(machine, "macos", StartReason::OwnMachineCapacity)
}

fn mac_runner_start(machine: &str, name: &str, reason: StartReason) -> Action {
    let mut action = start(
        "a",
        name,
        small(),
        Destination::Machine(MachineId(machine.into())),
        reason,
    );
    if let Action::StartRunner(start) = &mut action {
        start.platform = Platform::MacosArm64;
        // A macOS VM is an APFS clone: it holds its label disk and a margin.
        start.size.disk_bytes = start.requested_size.disk_bytes + MACOS_ROOT_MARGIN_BYTES;
    }
    action
}

#[test]
fn macos_jobs_run_only_on_macs() {
    let mut state = with_mac_label();
    state.machines.push(mac("mac"));
    state.queue = vec![queued("a", "macos", 100)];
    assert_eq!(decide(&state), vec![mac_start("mac")]);
}

#[test]
fn linux_jobs_never_run_on_macs() {
    let mut state = with_mac_label();
    state.machines = vec![mac("mac")];
    state.queue = vec![queued("a", "ciproxy", 100)];
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "ciproxy",
            small(),
            "small",
            StartReason::NoEligibleMachine,
        )]
    );
    state.labels[0].destinations.boat = false;
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn macos_jobs_wait_for_a_mac_whatever_the_boat_switch_says() {
    let mut state = with_mac_label();
    state.labels[2].destinations.boat = true;
    state.labels[2].nothing_fits = NothingFits::LargestBoatSize;
    state.queue = vec![queued("a", "macos", 100)];
    assert_eq!(decide(&state), vec![], "a free Linux machine only");

    state.machines = vec![mac("busy")];
    state.machines[0].capacity.0 = 0;
    assert_eq!(decide(&state), vec![], "a full Mac only");
}

#[test]
fn a_mac_runs_at_most_two_vms_whatever_its_slots() {
    let mut state = with_mac_label();
    state.machines = vec![mac("mac")];
    state.queue = vec![
        queued("a", "macos", 100),
        queued("a", "macos", 90),
        queued("a", "macos", 80),
    ];
    assert_eq!(decide(&state), vec![mac_start("mac"), mac_start("mac")]);

    let running = |id: &str| runner(id, "a", "macos", RunnerState::Running(JobId(id.into())));
    state.machines[0].runners = vec![running("first")];
    assert_eq!(decide(&state), vec![mac_start("mac")]);

    state.machines[0].runners.push(running("second"));
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn macs_keep_warm_runners_of_ciproxy_macos_only() {
    let mut state = snapshot();
    let mut warm = label("ciproxy-macos", small());
    warm.platform = Platform::MacosArm64;
    state.labels.push(warm);
    let mut machine = mac("mac");
    machine.warm_count = 3;
    machine.warm_allocation = vec![OrganizationId("a".into()); 3];
    state.machines = vec![machine];
    let expected = mac_runner_start("mac", "ciproxy-macos", StartReason::WarmPool);
    assert_eq!(decide(&state), vec![expected.clone(), expected]);

    state.labels.pop();
    assert_eq!(decide(&state), vec![], "no ciproxy-macos label");
}

/// The standard macOS release: a 100 GB disk with 21.8 GB free for a job.
fn standard_macos_image() -> ImageMetadata {
    ImageMetadata {
        image_id: ImageId("macos-26.7.3-standard-v2".into()),
        root_disk_bytes: 100 * GIGABYTE,
        free_job_bytes: Some(21_800_000_000),
    }
}

/// A Mac with 30 GB of job disk: its VM is an APFS clone that holds the
/// label disk and a margin (24 GB), not the 100 GB image disk.
#[test]
fn a_mac_with_little_job_disk_runs_a_macos_job() {
    let mut state = with_mac_label();
    let mut machine = mac("mac");
    machine.disk_budget_bytes = 30 * GIGABYTE;
    state.machines = vec![machine];
    state.queue = vec![queued("a", "macos", 100)];

    assert_eq!(decide(&state), vec![mac_start("mac")]);
}

/// A clone cannot give the guest more than the free space of the fixed
/// image disk, so a label that asks for more does not fit the Mac.
#[test]
fn a_macos_label_above_the_free_space_of_the_image_waits() {
    let mut state = with_mac_label();
    state.labels.last_mut().unwrap().size.disk_bytes = 30 * GIGABYTE;
    state.machines = vec![mac("mac")];
    state.queue = vec![queued("a", "macos", 100)];

    assert_eq!(decide(&state), vec![]);
}

/// An older agent reports no free bytes of its image, so its Mac gets no
/// new job until it does.
#[test]
fn a_mac_with_no_measured_image_free_bytes_gets_no_new_job() {
    let mut state = with_mac_label();
    let mut machine = mac("mac");
    machine.selected_image.as_mut().unwrap().free_job_bytes = None;
    state.machines = vec![machine];
    state.queue = vec![queued("a", "macos", 100)];

    assert_eq!(decide(&state), vec![]);
}

/// The agent refuses a VM that the real free space of its volume cannot
/// hold, so placement passes over a Mac whose running clones filled it.
#[test]
fn a_full_mac_passes_the_job_to_a_mac_with_real_free_space() {
    let mut state = with_mac_label();
    let mut full = mac("full");
    full.free_bytes = Some(20 * GIGABYTE);
    let mut roomy = mac("roomy");
    roomy.free_bytes = Some(60 * GIGABYTE);
    state.machines = vec![full, roomy];
    state.queue = vec![queued("a", "macos", 100), queued("a", "macos", 99)];

    assert_eq!(
        decide(&state),
        vec![mac_start("roomy"), mac_start("roomy")],
        "60 GB holds two VMs of 24 GB, and 20 GB holds none"
    );
}
