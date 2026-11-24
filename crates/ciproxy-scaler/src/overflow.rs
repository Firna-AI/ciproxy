//! Walk the label order for demand that has no accepted runner.

use crate::actions::{Destination, StartReason};
use crate::lifecycle::{admit_provider_start, claim_unsent};
use crate::placement::place_own;
use crate::provider::{ProviderAccount, ProviderCheck, ProviderSize};
use crate::size_policy::place_fits;
use crate::snapshot::{Label, NothingFits, Place, Snapshot, WaitingJob};
use crate::state::Plan;

pub(crate) enum Placement {
    Placed,
    CheckingProvider,
    Pending,
}

pub(crate) fn place_in_order(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
) -> Placement {
    let own_fits = place_fits(snapshot, label, Place::OwnMachines);
    let largest_allowed = label.nothing_fits == NothingFits::LargestProviderSize
        && !label
            .places()
            .any(|place| place_fits(snapshot, label, place));
    let mut before_own_machines = true;
    let mut reason = if own_fits {
        StartReason::ProviderBeforeOwnMachines
    } else {
        StartReason::NoEligibleMachine
    };

    for place in label.places() {
        match place {
            Place::OwnMachines => {
                before_own_machines = false;
                if place_own(snapshot, plan, job, label) {
                    return Placement::Placed;
                }
                if own_fits {
                    let waited = snapshot.now.saturating_sub(job.queued_at);
                    if waited < snapshot.overflow_wait {
                        return Placement::Pending;
                    }
                    reason = StartReason::OverflowWaitElapsed { waited };
                }
            }
            Place::Provider(id) if snapshot.overflow_enabled => {
                let Some(provider) = snapshot
                    .provider(id)
                    .filter(|provider| provider.connected_geometry().is_some())
                else {
                    continue;
                };
                if provider.check == ProviderCheck::InProgress && before_own_machines {
                    keep_unsent_during_check(snapshot, plan, job, label);
                    return Placement::CheckingProvider;
                }
                if claim_unsent(snapshot, plan, job, provider) {
                    return Placement::Placed;
                }
                if provider.check != ProviderCheck::Current {
                    continue;
                }
                if let Some(size) = provider.fitting_size(label.size)
                    && start_provider(snapshot, plan, job, label, provider, size, reason.clone())
                {
                    return Placement::Placed;
                }
                if largest_allowed
                    && let Some(size) = provider.largest_for_start(label.size)
                    && start_provider(
                        snapshot,
                        plan,
                        job,
                        label,
                        provider,
                        size,
                        StartReason::LargestProviderSize,
                    )
                {
                    return Placement::Placed;
                }
            }
            Place::Provider(_) => {}
        }
    }

    Placement::Pending
}

fn keep_unsent_during_check(snapshot: &Snapshot, plan: &mut Plan, job: &WaitingJob, label: &Label) {
    for place in label.places() {
        if let Place::Provider(id) = place
            && let Some(provider) = snapshot.provider(id)
            && provider.check == ProviderCheck::InProgress
            && claim_unsent(snapshot, plan, job, provider)
        {
            return;
        }
    }
}

fn start_provider(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
    provider: &ProviderAccount,
    size: &ProviderSize,
    reason: StartReason,
) -> bool {
    if !admit_provider_start(snapshot, plan, provider, size.start_bound) {
        return false;
    }
    plan.start(
        job,
        label,
        size.size,
        Destination::Provider {
            provider: provider.id,
            size_id: size.id.clone(),
        },
        reason,
    );
    true
}
