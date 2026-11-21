//! Reconciliation keeps demand, running jobs, and cleanup separate.

use ciproxy_protocol::identifiers::{OrganizationId, RunnerId};
use ciproxy_protocol::resources::{Slots, VmSize};

use crate::actions::{Action, StartReason, StopReason};
use crate::boat::{Refusal, SandboxId};
use crate::decision::decide;
use crate::snapshot::RunnerState;

use super::fixtures::{
    allocated, boat, boat_with_request, large, own, queued, runner, running, sandbox, small,
    snapshot, warm,
};

#[test]
fn oversized_boat_capacity_does_not_hide_a_label_size_change() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 0)];
    let mut oversized = runner("oversized", "a", "ciproxy", RunnerState::Ready);
    oversized.size = allocated(large());
    state.sandboxes = vec![sandbox("oversized", Some(oversized))];
    assert_eq!(decide(&state), vec![]);
    state.labels[0].size = VmSize { vcpu: 3, ..small() };
    assert_eq!(
        decide(&state),
        vec![
            Action::DeleteSandbox {
                sandbox_id: SandboxId("oversized".into()),
                reason: StopReason::LabelChanged
            },
            boat_with_request(
                "a",
                "ciproxy",
                VmSize { vcpu: 3, ..small() },
                large(),
                "large",
                StartReason::NoEligibleMachine
            ),
        ]
    );
}

#[test]
fn counts_starting_and_ready_runners_once_per_organization_and_label() {
    for phase in [RunnerState::Starting, RunnerState::Ready] {
        let mut state = snapshot();
        state.queue = vec![queued("a", "ciproxy", 100), queued("a", "ciproxy", 100)];
        state.machines[0].runners = vec![runner("pending", "a", "ciproxy", phase.clone())];
        assert_eq!(
            decide(&state),
            vec![own("a", "ciproxy", small())],
            "{phase:?}"
        );
        state.queue = vec![queued("a", "ciproxy", 100), queued("b", "ciproxy", 100)];
        assert_eq!(
            decide(&state),
            vec![own("b", "ciproxy", small())],
            "{phase:?}"
        );
    }
}

#[test]
fn counts_starting_boat_runners_without_duplicate_overflow() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 100)];
    state.sandboxes = vec![sandbox(
        "pending",
        Some(runner("runner", "a", "ciproxy", RunnerState::Starting)),
    )];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn an_idle_own_runner_makes_an_unneeded_boat_runner_retire() {
    let mut state = snapshot();
    state.queue = vec![queued("a", "ciproxy", 100)];
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into())];
    state.machines[0].runners = vec![warm("warm", "a")];
    state.sandboxes = vec![sandbox(
        "idle",
        Some(runner("boat", "a", "ciproxy", RunnerState::Ready)),
    )];
    assert_eq!(
        decide(&state),
        vec![Action::DeleteSandbox {
            sandbox_id: SandboxId("idle".into()),
            reason: StopReason::NoWaitingJob,
        }]
    );
}

#[test]
fn size_change_retires_old_idle_runners_and_preserves_running_jobs() {
    let mut state = snapshot();
    state.labels[0].size = VmSize { vcpu: 4, ..small() };
    state.queue = vec![queued("a", "ciproxy", 0)];
    state.machines[0].runners = vec![
        runner("starting", "a", "ciproxy", RunnerState::Starting),
        runner("ready", "a", "ciproxy", RunnerState::Ready),
        running("running"),
    ];
    state.sandboxes = vec![
        sandbox(
            "old",
            Some(runner("boat-idle", "a", "ciproxy", RunnerState::Ready)),
        ),
        sandbox("busy", Some(running("boat-running"))),
    ];
    assert_eq!(
        decide(&state),
        vec![
            Action::StopRunner {
                runner_id: RunnerId("starting".into()),
                reason: StopReason::LabelChanged
            },
            Action::StopRunner {
                runner_id: RunnerId("ready".into()),
                reason: StopReason::LabelChanged
            },
            Action::DeleteSandbox {
                sandbox_id: SandboxId("old".into()),
                reason: StopReason::LabelChanged
            },
            own("a", "ciproxy", VmSize { vcpu: 4, ..small() }),
        ]
    );
}

#[test]
fn retiring_runners_keep_their_slots_and_disk_until_removed() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(1);
    state.machines[0].runners = vec![runner("retiring", "a", "ciproxy", RunnerState::Retiring)];
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(decide(&state), vec![]);
    state.machines[0].runners.clear();
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
}

#[test]
fn removing_an_organization_stops_only_its_idle_runners() {
    let mut state = snapshot();
    state.organizations = vec![OrganizationId("b".into())];
    state.queue = vec![queued("a", "ciproxy", 100), queued("b", "ciproxy", 0)];
    state.machines[0].runners = vec![warm("idle-a", "a"), running("busy-a")];
    state.sandboxes = vec![
        sandbox(
            "idle-a",
            Some(runner("boat-idle", "a", "ciproxy", RunnerState::Ready)),
        ),
        sandbox("busy-a", Some(running("boat-busy"))),
    ];
    assert_eq!(
        decide(&state),
        vec![
            Action::StopRunner {
                runner_id: RunnerId("idle-a".into()),
                reason: StopReason::OrganizationRemoved
            },
            Action::DeleteSandbox {
                sandbox_id: SandboxId("idle-a".into()),
                reason: StopReason::OrganizationRemoved
            },
            own("b", "ciproxy", small()),
        ]
    );
}

#[test]
fn cleans_only_marked_idle_sandboxes_and_always_deletes_exited_runners() {
    let mut state = snapshot();
    state.sandboxes = vec![
        sandbox("orphan", None),
        sandbox(
            "idle",
            Some(runner("idle", "a", "ciproxy", RunnerState::Ready)),
        ),
        sandbox(
            "exited",
            Some(runner("exited", "a", "ciproxy", RunnerState::Exited)),
        ),
        sandbox("running", Some(running("running"))),
        sandbox("foreign", None),
    ];
    state.sandboxes[4].marked = false;
    assert_eq!(
        decide(&state),
        vec![
            Action::DeleteSandbox {
                sandbox_id: SandboxId("orphan".into()),
                reason: StopReason::OrphanSandbox
            },
            Action::DeleteSandbox {
                sandbox_id: SandboxId("exited".into()),
                reason: StopReason::RunnerExited
            },
            Action::DeleteSandbox {
                sandbox_id: SandboxId("idle".into()),
                reason: StopReason::NoWaitingJob
            },
        ]
    );
}

#[test]
fn completed_sandbox_is_deleted_even_when_more_jobs_wait() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 0)];
    state.sandboxes = vec![sandbox(
        "done",
        Some(runner("exited", "a", "ciproxy", RunnerState::Exited)),
    )];
    assert_eq!(
        decide(&state),
        vec![
            Action::DeleteSandbox {
                sandbox_id: SandboxId("done".into()),
                reason: StopReason::RunnerExited
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
}

#[test]
fn failures_never_retire_or_retry_running_jobs() {
    for case in [
        "offline",
        "draining",
        "disconnected",
        "rejected-key",
        "spend-limit",
        "label-removed",
        "target-unknown",
    ] {
        let mut state = snapshot();
        state.machines[0].runners = vec![running("machine-busy")];
        state.sandboxes = vec![sandbox("boat-busy", Some(running("boat-busy")))];
        match case {
            "offline" => state.machines[0].connected = false,
            "draining" => state.machines[0].draining = true,
            "disconnected" => state.boat.connected = false,
            "rejected-key" => state.boat.refusals = vec![Refusal::RejectedKey],
            "spend-limit" => state.boat.spend.spent = 100,
            "label-removed" => state.labels.clear(),
            "target-unknown" => {
                state.machines[0].runners[0].target = None;
                state.sandboxes[0].runner.as_mut().unwrap().runner.target = None;
            }
            _ => unreachable!(),
        }
        assert_eq!(decide(&state), vec![], "{case}");
    }
}

#[test]
fn control_plane_outage_produces_no_changes() {
    let mut state = snapshot();
    state.control_plane_available = false;
    state.queue = vec![queued("a", "ciproxy", 100)];
    state.machines[0].runners = vec![running("busy")];
    state.sandboxes = vec![sandbox("orphan", None)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn pending_start_in_the_next_snapshot_prevents_a_duplicate_tick() {
    let mut state = snapshot();
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
    let pending = runner("persisted", "a", "ciproxy", RunnerState::Starting);
    state.machines[0].runners.push(pending);
    assert_eq!(decide(&state), vec![]);
}
