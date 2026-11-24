//! New starts walk the label order across machines and providers.

use std::time::Duration;

use ciproxy_protocol::identifiers::{MachineId, RunnerId};
use ciproxy_protocol::resources::Slots;

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::decision::decide;
use crate::provider::{Refusal, SandboxId};
use crate::snapshot::{Place, RunnerState};

use super::fixtures::{
    BOAT, SECOND, large, own, provider_destination, queued, runner, running, sandbox, small, start,
    two_provider_snapshot, warm,
};

#[test]
fn first_fitting_provider_wins_even_when_listed_later_or_more_expensive() {
    let mut state = two_provider_snapshot();
    state.labels[0].order = vec![Place::Provider(SECOND), Place::Provider(BOAT)];
    state.providers[1].facts.sizes[0].price = 3;
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            provider_destination(SECOND, "small"),
            StartReason::NoEligibleMachine,
        )],
    );
}

#[test]
fn skips_each_provider_start_block_without_waiting() {
    for case in [
        "missing",
        "disconnected",
        "unavailable",
        "rejected key",
        "spend limit",
        "retry time",
        "billing",
        "no fitting size",
        "refused size",
    ] {
        let mut state = two_provider_snapshot();
        state.labels[1].order = vec![Place::Provider(BOAT), Place::Provider(SECOND)];
        state.queue = vec![queued("a", "large", 0)];
        let first = &mut state.providers[0];
        match case {
            "missing" => {
                state.providers.remove(0);
            }
            "disconnected" => first.facts.connected = false,
            "unavailable" => first.start_geometry = None,
            "rejected key" => first.facts.refusals.push(Refusal::RejectedKey),
            "spend limit" => first.facts.spend.spent = 100,
            "retry time" => first.facts.refusals.push(Refusal::RetryAfter {
                not_before: Duration::from_secs(101),
            }),
            "billing" => first.facts.refusals.push(Refusal::BillingRequired),
            "no fitting size" => first.facts.sizes.truncate(1),
            "refused size" => first
                .facts
                .refusals
                .push(Refusal::SizeNotOnPlan(first.facts.sizes[1].id.clone())),
            _ => unreachable!(),
        }
        assert_eq!(
            decide(&state),
            vec![start(
                "a",
                "large",
                large(),
                provider_destination(SECOND, "large"),
                StartReason::NoEligibleMachine,
            )],
            "{case}",
        );
    }
}

#[test]
fn provider_runs_before_free_own_machines_and_can_fall_back_to_them() {
    let mut state = two_provider_snapshot();
    state.labels[0].order = vec![
        Place::Provider(BOAT),
        Place::OwnMachines,
        Place::Provider(SECOND),
    ];
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            provider_destination(BOAT, "small"),
            StartReason::ProviderBeforeOwnMachines,
        )],
    );
    state.providers[0].facts.connected = false;
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
    state.machines[0].runners = vec![runner("accepted", "a", "ciproxy", RunnerState::Starting)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn leaving_own_machines_out_keeps_free_machines_out_of_routing() {
    let mut state = two_provider_snapshot();
    state.labels[0].order = vec![Place::Provider(BOAT), Place::Provider(SECOND)];
    state.providers[0].facts.connected = false;
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            provider_destination(SECOND, "small"),
            StartReason::NoEligibleMachine,
        )],
    );
}

#[test]
fn waits_only_after_reaching_busy_own_machines_that_fit() {
    let mut state = two_provider_snapshot();
    state.labels[0].order = vec![
        Place::Provider(BOAT),
        Place::OwnMachines,
        Place::Provider(SECOND),
    ];
    state.providers[0]
        .facts
        .refusals
        .push(Refusal::BillingRequired);
    state.machines[0].capacity = Slots(1);
    state.machines[0].runners = vec![running("busy")];
    state.queue = vec![queued("a", "ciproxy", 29)];
    assert_eq!(decide(&state), vec![]);
    state.queue = vec![queued("a", "ciproxy", 30)];
    let second = |reason| {
        start(
            "a",
            "ciproxy",
            small(),
            provider_destination(SECOND, "small"),
            reason,
        )
    };
    assert_eq!(
        decide(&state),
        vec![second(StartReason::OverflowWaitElapsed {
            waited: Duration::from_secs(30),
        })],
    );
    state.machines[0].connected = false;
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(decide(&state), vec![second(StartReason::NoEligibleMachine)]);
}

#[test]
fn workspace_switch_skips_every_provider_even_before_own_machines() {
    let mut state = two_provider_snapshot();
    state.labels[0].order = vec![
        Place::Provider(BOAT),
        Place::Provider(SECOND),
        Place::OwnMachines,
    ];
    state.queue = vec![queued("a", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        vec![start(
            "a",
            "ciproxy",
            small(),
            provider_destination(BOAT, "small"),
            StartReason::ProviderBeforeOwnMachines,
        )],
    );
    state.overflow_enabled = false;
    assert_eq!(decide(&state), vec![own("a", "ciproxy", small())]);
    state.labels[0].order.pop();
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn stops_unused_warm_runners_and_fills_the_pool_only_with_own_machines_first() {
    let mut state = two_provider_snapshot();
    state.machines[0].warm_count = 2;
    state.machines[0].warm_allocation = state.organizations.clone();
    let warm_start = |org| {
        start(
            org,
            "ciproxy",
            small(),
            Destination::Machine(MachineId("machine".into())),
            StartReason::WarmPool,
        )
    };
    assert_eq!(decide(&state), vec![warm_start("a"), warm_start("b")]);
    state.machines[0].runners = vec![warm("idle", "a"), running("busy")];
    state.labels[0].order.swap(0, 1);
    state.overflow_enabled = false;
    assert_eq!(
        decide(&state),
        vec![Action::StopRunner {
            runner_id: RunnerId("idle".into()),
            reason: StopReason::NoWaitingJob,
        }],
    );
    state.machines[0].runners.remove(0);
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn equal_sandbox_ids_on_two_providers_get_separate_deletions() {
    let mut state = two_provider_snapshot();
    let first = sandbox("orphan", None);
    let mut second = first.clone();
    second.provider = SECOND;
    state.sandboxes = vec![first, second];
    assert_eq!(
        decide(&state),
        [BOAT, SECOND].map(|provider| Action::DeleteSandbox {
            provider,
            sandbox_id: SandboxId("orphan".into()),
            reason: StopReason::OrphanSandbox,
        }),
    );
}
