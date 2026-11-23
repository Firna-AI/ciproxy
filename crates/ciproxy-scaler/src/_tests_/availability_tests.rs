//! Account availability, spend limits, and provider retry facts.

use std::time::Duration;

use crate::actions::StartReason;
use crate::decision::decide;
use crate::provider::{ProviderSizeId, Refusal, StartBound};
use crate::snapshot::Place;

use super::fixtures::{BOAT, boat, boat_with_request, large, own, queued, small, snapshot};

#[test]
fn boat_availability_blocks_new_starts_at_the_spend_limit() {
    for case in [
        "off",
        "disconnected",
        "at-limit",
        "above-limit",
        "key",
        "billing",
    ] {
        let mut state = snapshot();
        state.machines.clear();
        state.queue = vec![queued("a", "ciproxy", 100)];
        match case {
            "off" => state.overflow_enabled = false,
            "disconnected" => state.providers[0].facts.connected = false,
            "at-limit" => state.providers[0].facts.spend.spent = 100,
            "above-limit" => state.providers[0].facts.spend.spent = 101,
            "key" => state.providers[0].facts.refusals = vec![Refusal::RejectedKey],
            "billing" => state.providers[0].facts.refusals = vec![Refusal::BillingRequired],
            _ => unreachable!(),
        }
        assert_eq!(decide(&state), vec![], "{case}");
        state.machines = snapshot().machines;
        assert_eq!(decide(&state), vec![own("a", "ciproxy", small())], "{case}");
    }
}

#[test]
fn raised_or_reset_spend_limit_allows_new_overflow() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 0)];
    for (spent, limit) in [(100, 101), (0, 100), (99, 100)] {
        state.providers[0].facts.spend.spent = spent;
        state.providers[0].facts.spend.limit = limit;
        assert_eq!(
            decide(&state),
            vec![boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::NoEligibleMachine
            )]
        );
    }
}

#[test]
fn starts_in_the_same_decision_reserve_the_next_months_budget() {
    let mut state = snapshot();
    state.labels[0].order = vec![Place::Provider(BOAT), Place::OwnMachines];
    state.providers[0].facts.spend.next_month_spent = Some(35);
    state.providers[0].facts.sizes[0].start_bound = Some(StartBound {
        current_month: 5,
        next_month: Some(50),
    });
    state.queue = vec![queued("a", "ciproxy", 0), queued("b", "ciproxy", 0)];
    assert_eq!(
        decide(&state),
        [
            boat(
                "a",
                "ciproxy",
                small(),
                "small",
                StartReason::ProviderBeforeOwnMachines
            ),
            own("b", "ciproxy", small()),
        ]
    );
}

#[test]
fn retries_at_the_backend_time_and_excludes_sizes_refused_by_the_plan() {
    let mut state = snapshot();
    state.machines.clear();
    state.queue = vec![queued("a", "ciproxy", 100)];
    state.providers[0].facts.refusals = vec![Refusal::RetryAfter {
        not_before: Duration::from_secs(101),
    }];
    assert_eq!(decide(&state), vec![]);
    state.now = Duration::from_secs(101);
    assert_eq!(
        decide(&state),
        vec![boat(
            "a",
            "ciproxy",
            small(),
            "small",
            StartReason::NoEligibleMachine
        )]
    );
    state.providers[0]
        .facts
        .refusals
        .push(Refusal::SizeNotOnPlan(ProviderSizeId("small".into())));
    assert_eq!(
        decide(&state),
        vec![boat_with_request(
            "a",
            "ciproxy",
            small(),
            large(),
            "large",
            StartReason::NoEligibleMachine
        )]
    );
}
