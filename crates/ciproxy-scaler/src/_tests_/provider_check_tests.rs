//! Provider check states keep waiting demand from preempting warm capacity early.

use ciproxy_protocol::identifiers::{OrganizationId, RunnerId};
use ciproxy_protocol::resources::Slots;

use crate::actions::{Action, StopReason};
use crate::decision::decide;
use crate::provider::{ProviderCheck, ProviderDispatch, Refusal, SandboxId, StartBound};
use crate::snapshot::{RunnerState, Snapshot};

use super::fixtures::{BOAT, own, queued, runner, sandbox, small, snapshot, warm};

fn provider_first_with_warm_pool() -> Snapshot {
    let mut state = snapshot();
    state.labels[1].order.swap(0, 1);
    state.machines[0].capacity = Slots(2);
    state.machines[0].warm_count = 1;
    state.machines[0].warm_allocation = vec![OrganizationId("a".into())];
    state.machines[0].runners = vec![warm("warm", "a")];
    state.queue = vec![queued("a", "large", 30)];
    state
}

#[test]
fn provider_check_in_progress_parks_demand_and_keeps_the_warm_runner() {
    let mut state = provider_first_with_warm_pool();
    state.providers[0].check = ProviderCheck::InProgress;
    assert_eq!(decide(&state), []);

    state.machines[0].runners.clear();
    assert_eq!(decide(&state), []);

    state.labels[0].order.truncate(1);
    state.queue.push(queued("a", "ciproxy", 0));
    assert_eq!(decide(&state), [own("a", "ciproxy", small())]);
}

#[test]
fn held_provider_allows_waiting_demand_to_preempt_a_warm_runner() {
    let mut state = provider_first_with_warm_pool();
    state.providers[0]
        .facts
        .refusals
        .push(Refusal::BillingRequired);
    assert_eq!(
        decide(&state),
        [Action::StopRunner {
            runner_id: RunnerId("warm".into()),
            reason: StopReason::WarmPreemption {
                label_id: "large".into()
            },
        }]
    );
}

fn provider_first_with_unsent() -> Snapshot {
    let mut state = snapshot();
    state.labels[0].order.swap(0, 1);
    state.queue = vec![queued("a", "ciproxy", 0)];
    let mut pending = sandbox(
        "pending",
        Some(runner("pending", "a", "ciproxy", RunnerState::Starting)),
    );
    pending.runner.as_mut().unwrap().dispatch = ProviderDispatch::Unsent(Some(StartBound {
        current_month: 30,
        next_month: None,
    }));
    state.sandboxes = vec![pending];
    state
}

#[test]
fn current_provider_check_selects_the_unsent_start_after_the_check() {
    let mut state = provider_first_with_unsent();
    state.providers[0].check = ProviderCheck::InProgress;
    assert_eq!(decide(&state), []);

    state.providers[0].check = ProviderCheck::Current;
    assert_eq!(
        decide(&state),
        [Action::ResumeProviderStart {
            runner_id: RunnerId("pending".into())
        }]
    );
}

#[test]
fn unsent_start_releases_demand_when_overflow_is_off_held_or_over_budget() {
    let Action::StartRunner(start) = own("a", "ciproxy", small()) else {
        panic!("expected an own-machine start");
    };
    for (case, state) in [
        ("overflow off", {
            let mut state = provider_first_with_unsent();
            state.overflow_enabled = false;
            state
        }),
        ("provider held", {
            let mut state = provider_first_with_unsent();
            state.providers[0]
                .facts
                .refusals
                .push(Refusal::BillingRequired);
            state
        }),
        ("start exceeds the budget", {
            let mut state = provider_first_with_unsent();
            state.providers[0].facts.spend.spent = 71;
            for size in &mut state.providers[0].facts.sizes {
                size.start_bound = Some(StartBound {
                    current_month: 30,
                    next_month: None,
                });
            }
            state
        }),
    ] {
        assert_eq!(
            decide(&state),
            [
                Action::ReplaceUnsentStart {
                    runner_id: RunnerId("pending".into()),
                    start: start.clone()
                },
                Action::DeleteSandbox {
                    provider: BOAT,
                    sandbox_id: SandboxId("pending".into()),
                    reason: StopReason::NoWaitingJob
                }
            ],
            "{case}"
        );
    }
}
