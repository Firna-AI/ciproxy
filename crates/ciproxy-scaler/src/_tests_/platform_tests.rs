//! A job runs only on a machine of its label's platform, and macOS never uses Boat.

use ciproxy_protocol::identifiers::{JobId, MachineId, OrganizationId};
use ciproxy_protocol::platform::Platform;

use crate::actions::{Action, Destination, StartReason};
use crate::decision::decide;
use crate::snapshot::{Machine, NothingFits, RunnerState, Snapshot};

use super::fixtures::{boat, label, queued, runner, small, snapshot, start};

fn mac(id: &str) -> Machine {
    let mut machine = snapshot().machines.remove(0);
    machine.id = MachineId(id.into());
    machine.platform = Platform::MacosArm64;
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
