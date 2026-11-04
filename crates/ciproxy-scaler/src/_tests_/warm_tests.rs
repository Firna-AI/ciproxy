//! Shared warm allocation, preemption, and queue precedence.

use std::time::Duration;

use ciproxy_protocol::identifiers::{MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::resources::Slots;

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::decision::decide;
use crate::snapshot::RunnerState;

use super::fixtures::{boat, large, own, queued, running, small, snapshot, start, warm};

fn warm_start(organization: &str) -> Action {
    start(
        organization,
        "ciproxy",
        small(),
        Destination::Machine(MachineId("machine".into())),
        StartReason::WarmPool,
    )
}

#[test]
fn one_machine_count_caps_the_recent_job_allocation_across_organizations() {
    let mut state = snapshot();
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation = vec![
        OrganizationId("a".into()),
        OrganizationId("b".into()),
        OrganizationId("a".into()),
    ];
    assert_eq!(decide(&state), vec![warm_start("a"), warm_start("b")]);
}

#[test]
fn replenishes_a_warm_runner_only_after_it_takes_a_job() {
    let mut state = snapshot();
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    let mut pending = warm("pending", "a");
    pending.state = RunnerState::Starting;
    state.machines[0].runners = vec![pending, warm("ready", "b")];
    assert_eq!(decide(&state), vec![]);
    state.machines[0].runners[1].state = running("job").state;
    assert_eq!(decide(&state), vec![warm_start("b")]);
}

#[test]
fn retires_a_changed_allocation_before_replacing_its_warm_runner() {
    let mut state = snapshot();
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![warm("a-1", "a"), warm("a-2", "a")];
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("a-2".into()),
            reason: StopReason::WarmAllocationChanged,
        }]
    );
    state.machines[0].runners.pop();
    assert_eq!(decide(&state), vec![warm_start("b")]);
}

#[test]
fn stops_only_the_warm_capacity_a_waiting_job_needs_then_waits_for_release() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(3);
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
    state.machines[0].runners.remove(0);
    assert_eq!(decide(&state), vec![own("a", "large", large())]);
}

#[test]
fn leaves_free_capacity_for_a_waiting_job_that_needs_more_than_is_free() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(2);
    state.machines[0].runners = vec![running("busy")];
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into())];
    state.queue = vec![queued("a", "large", 100)];
    state.boat.enabled = false;
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn starts_waiting_jobs_before_filling_the_warm_pool() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(2);
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.queue = vec![queued("a", "large", 0)];
    assert_eq!(decide(&state), vec![own("a", "large", large())]);
}

#[test]
fn draining_retires_idle_warm_runners_and_keeps_running_jobs() {
    let mut state = snapshot();
    state.machines[0].draining = true;
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![warm("idle", "a"), running("busy")];
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("idle".into()),
            reason: StopReason::MachineDraining,
        }]
    );
}

#[test]
fn an_offline_ready_warm_runner_can_still_cover_github_demand() {
    let mut state = snapshot();
    state.machines[0].connected = false;
    state.machines[0].runners = vec![warm("ready", "a"), running("busy")];
    state.queue = vec![queued("a", "ciproxy", 100)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn warm_pool_uses_only_the_ciproxy_label_and_active_organizations() {
    let mut state = snapshot();
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("gone".into()), OrganizationId("b".into())];
    assert_eq!(decide(&state), vec![warm_start("b")]);
    state.labels.remove(0);
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn delayed_warm_retirement_does_not_extend_the_overflow_wait() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(2);
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into())];
    let mut retiring = warm("retiring", "a");
    retiring.state = RunnerState::Retiring;
    state.machines[0].runners = vec![retiring];
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

#[test]
fn reserves_distinct_future_capacity_for_each_waiting_job() {
    let mut state = snapshot();
    state.machines[0].warm_count = 6;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into()); 6];
    state.machines[0].runners = vec![
        warm("one", "a"),
        warm("two", "a"),
        warm("three", "a"),
        warm("four", "a"),
        warm("five", "a"),
        warm("six", "a"),
    ];
    state.queue = vec![queued("a", "large", 0), queued("a", "large", 0)];
    assert_eq!(
        decide(&state),
        vec![
            Action::StopRunner {
                runner_id: RunnerId("one".into()),
                reason: StopReason::WarmPreemption
            },
            Action::StopRunner {
                runner_id: RunnerId("two".into()),
                reason: StopReason::WarmPreemption
            },
            Action::StopRunner {
                runner_id: RunnerId("three".into()),
                reason: StopReason::WarmPreemption
            },
            Action::StopRunner {
                runner_id: RunnerId("four".into()),
                reason: StopReason::WarmPreemption
            },
        ]
    );
}

#[test]
fn boat_covered_demand_does_not_preempt_the_own_warm_pool() {
    let mut state = snapshot();
    state.machines[0].capacity = Slots(2);
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation =
        vec![OrganizationId("a".into()), OrganizationId("b".into())];
    state.machines[0].runners = vec![warm("one", "a"), warm("two", "b")];
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
