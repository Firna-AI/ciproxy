//! A smaller provider size can start only when the full request fits nowhere.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::{Action, StartReason, StartRunner, StopReason};
use crate::decision::decide;
use crate::provider::{ProviderId, ProviderSizeId, Refusal, SandboxId};
use crate::snapshot::{NothingFits, Place, RunnerState, Snapshot};

use super::fixtures::{
    BOAT, SECOND, allocated, boat_size, large, provider_destination, queued, runner, running,
    sandbox, target, two_provider_snapshot,
};

fn requested() -> VmSize {
    VmSize {
        vcpu: 16,
        ..large()
    }
}

fn fallback_snapshot() -> Snapshot {
    let mut state = two_provider_snapshot();
    state.labels[1].order = vec![Place::Provider(SECOND), Place::Provider(BOAT)];
    state.labels[1].size = requested();
    state.labels[1].nothing_fits = NothingFits::LargestProviderSize;
    state.queue = vec![queued("a", "large", 0)];
    state
}

fn overflow(provider: ProviderId, size_id: &str, size: VmSize, reason: StartReason) -> Action {
    Action::StartRunner(Box::new(StartRunner {
        platform: Platform::LinuxX64,
        target: target("a", "large"),
        organization_id: OrganizationId("a".into()),
        label: LabelName("large".into()),
        requested_size: requested(),
        size: allocated(size),
        destination: provider_destination(provider, size_id),
        reason,
    }))
}

fn fallback(provider: ProviderId) -> Action {
    overflow(provider, "large", large(), StartReason::LargestProviderSize)
}

#[test]
fn uses_the_largest_size_of_the_first_eligible_provider_in_the_order() {
    let mut state = fallback_snapshot();
    state.providers[0]
        .facts
        .sizes
        .push(boat_size("bigger", VmSize { vcpu: 8, ..large() }, 10));
    state.providers[0].facts.largest_size = Some(ProviderSizeId("bigger".into()));
    assert_eq!(decide(&state), vec![fallback(SECOND)]);
}

#[test]
fn skips_largest_sizes_that_cannot_start_or_hold_the_providers_image() {
    for case in [
        "disconnected",
        "unavailable",
        "spend limit",
        "billing",
        "image floor",
    ] {
        let mut state = fallback_snapshot();
        let first = &mut state.providers[1];
        match case {
            "disconnected" => first.facts.connected = false,
            "unavailable" => first.start_geometry = None,
            "spend limit" => first.facts.spend.spent = 100,
            "billing" => first.facts.refusals.push(Refusal::BillingRequired),
            "image floor" => {
                first.start_geometry = Some(RootSizing {
                    image_root_bytes: 100_000_000_000,
                })
            }
            _ => unreachable!(),
        }
        assert_eq!(decide(&state), vec![fallback(BOAT)], "{case}");
        state.providers[0].facts.connected = false;
        assert_eq!(decide(&state), vec![], "both unavailable: {case}");
    }
}

#[test]
fn a_later_provider_that_fits_blocks_largest_size_even_during_its_retry_hold() {
    let mut state = fallback_snapshot();
    state.providers[0]
        .facts
        .sizes
        .push(boat_size("full", requested(), 10));
    assert_eq!(
        decide(&state),
        vec![overflow(
            BOAT,
            "full",
            requested(),
            StartReason::NoEligibleMachine
        )],
    );
    state.providers[0].facts.refusals.push(Refusal::RetryAfter {
        not_before: Duration::from_secs(101),
    });
    assert_eq!(decide(&state), vec![]);
    state.providers[0].start_geometry = None;
    assert_eq!(decide(&state), vec![fallback(SECOND)]);
    state.overflow_enabled = false;
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn busy_own_machines_that_fit_later_in_the_order_block_largest_size() {
    let mut state = fallback_snapshot();
    state.machines[0].capacity = Slots(8);
    let mut busy = running("busy");
    busy.size = allocated(requested());
    state.machines[0].runners = vec![busy];
    assert_eq!(decide(&state), vec![fallback(SECOND)]);

    state.labels[1].order.push(Place::OwnMachines);
    state.queue = vec![queued("a", "large", 30)];
    assert_eq!(decide(&state), vec![]);
}

#[test]
fn an_idle_largest_size_retires_when_another_provider_fits() {
    let mut state = fallback_snapshot();
    let mut idle = runner("fallback", "a", "large", RunnerState::Ready);
    idle.requested_size = requested();
    idle.size = allocated(large());
    let mut retained = sandbox("idle", Some(idle));
    retained.provider = SECOND;
    state.sandboxes = vec![retained];
    assert_eq!(decide(&state), vec![]);

    state.providers[0]
        .facts
        .sizes
        .push(boat_size("full", requested(), 10));
    assert_eq!(
        decide(&state),
        vec![
            Action::DeleteSandbox {
                provider: SECOND,
                sandbox_id: SandboxId("idle".into()),
                reason: StopReason::SizeNoLongerAllowed,
            },
            overflow(BOAT, "full", requested(), StartReason::NoEligibleMachine),
        ],
    );
}

#[test]
fn an_idle_largest_size_covers_demand_without_a_start_image() {
    let mut state = fallback_snapshot();
    state.labels[1].order = vec![Place::Provider(BOAT)];
    state.labels[1].size = VmSize {
        disk_bytes: 35 * GIGABYTE,
        ..large()
    };
    let actual = VmSize {
        disk_bytes: 100 * GIGABYTE,
        ..large()
    };
    state.providers[0].start_geometry = None;
    state.providers[0].facts.sizes[1].size = actual;
    let mut idle = runner("fallback", "a", "large", RunnerState::Ready);
    idle.requested_size = state.labels[1].size;
    idle.size = actual;
    let mut retained = sandbox("idle", Some(idle));
    retained.runner.as_mut().unwrap().required_root_disk_bytes = 107 * GIGABYTE;
    state.sandboxes = vec![retained];

    assert_eq!(decide(&state), []);
}
