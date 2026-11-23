//! Unavailable Docker reservations preserve eligibility and existing supply.

use std::time::Duration;

use ciproxy_protocol::identifiers::{OrganizationId, RunnerId};
use ciproxy_protocol::resources::{GIGABYTE, Slots};

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::decision::decide;
use crate::snapshot::RunnerState;

use super::fixtures::{boat, large, own, queued, runner, running, small, snapshot, start, warm};

#[test]
fn unknown_leases_keep_the_overflow_wait_and_allow_own_starts_after_recovery() {
    let mut state = snapshot();
    state.machines[0].docker_leased_bytes = None;
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
                waited: Duration::from_secs(30),
            },
        )]
    );
    state.machines[0].docker_leased_bytes = Some(0);
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
}

#[test]
fn measured_leases_reduce_start_capacity_and_physical_fit_once() {
    for (leased_gb, expected) in [
        (
            13,
            vec![own("a", "ciproxy", small()), own("a", "ciproxy", small())],
        ),
        (40, vec![own("a", "ciproxy", small())]),
    ] {
        let mut state = snapshot();
        state.overflow_enabled = false;
        state.machines[0].disk_budget_bytes = 180 * GIGABYTE;
        state.machines[0].docker_leased_bytes = Some(leased_gb * GIGABYTE);
        state.queue = vec![queued("a", "ciproxy", 0); 3];
        assert_eq!(decide(&state), expected, "{leased_gb} GB reserved");
    }
    let mut state = snapshot();
    state.machines[0].disk_budget_bytes = 180 * GIGABYTE;
    state.machines[0].docker_leased_bytes = Some(100 * GIGABYTE);
    state.queue = vec![queued("a", "ciproxy", 0)];
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
}

#[test]
fn unknown_leases_block_new_warm_runners_until_measurement_recovers() {
    let mut state = snapshot();
    state.machines[0].docker_leased_bytes = None;
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into())];
    assert_eq!(decide(&state), vec![]);
    state.machines[0].docker_leased_bytes = Some(0);
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            Destination::Machine(state.machines[0].id.clone()),
            StartReason::WarmPool,
        )]
    );
}

#[test]
fn unknown_leases_preserve_ready_starting_and_running_supply() {
    let mut state = snapshot();
    state.machines[0].docker_leased_bytes = None;
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![
        warm("ready", "a"),
        runner("starting", "b", "ciproxy", RunnerState::Starting),
        running("busy"),
    ];
    state.queue = vec![queued("a", "ciproxy", 100), queued("b", "ciproxy", 100)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn unknown_leases_prevent_projected_starts_but_keep_warm_target_reconciliation() {
    let mut state = snapshot();
    state.machines[0].docker_leased_bytes = None;
    state.machines[0].capacity = Slots(3);
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![warm("first", "a"), warm("second", "b")];
    state.queue = vec![queued("a", "large", 0)];
    assert_eq!(decide(&state), vec![]);
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("b".into())];
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("first".into()),
            reason: StopReason::WarmAllocationChanged,
        }]
    );
    state.machines[0].docker_leased_bytes = Some(0);
    state.machines[0].runners.remove(0);
    assert_eq!(decide(&state), vec![own("a", "large", large())]);
}
