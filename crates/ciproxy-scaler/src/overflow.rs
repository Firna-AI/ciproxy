//! Walk the label order for demand that has no retained runner.

use std::time::Duration;

use crate::actions::{Destination, StartReason};
use crate::capacity::own_fits;
use crate::placement::place_own;
use crate::provider::{ProviderAccount, ProviderId, ProviderSize, Refusal};
use crate::snapshot::{Label, NothingFits, Place, Snapshot, WaitingJob};
use crate::state::Plan;

/// Remember a fallback candidate while checking later places for full fit.
pub(crate) fn place_in_order(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
) -> bool {
    let own_fits = own_fits(snapshot, label);
    let mut has_fit = own_fits;
    let mut fallback = None;
    let mut reason = if own_fits {
        StartReason::ProviderBeforeOwnMachines
    } else {
        StartReason::NoEligibleMachine
    };

    for place in label.places() {
        match place {
            Place::OwnMachines => {
                if place_own(snapshot, plan, job, label) {
                    return true;
                }
                if own_fits {
                    let waited = snapshot.now.saturating_sub(job.queued_at);
                    if waited < snapshot.overflow_wait {
                        return false;
                    }
                    reason = StartReason::OverflowWaitElapsed { waited };
                }
            }
            Place::Provider(id) if snapshot.overflow_enabled => {
                let Some(provider) = snapshot
                    .provider(id)
                    .filter(|provider| provider.connected_and_available())
                else {
                    continue;
                };
                let Some(required) = provider.root_sizing.vm_size(label.size) else {
                    continue;
                };
                let fitting = provider.fitting_size(required);
                has_fit |= fitting.is_some();
                if !provider.refreshed || starts_held(provider, snapshot.now) {
                    continue;
                }
                if let Some(size) = fitting
                    && start_provider(plan, job, label, id, size, reason.clone())
                {
                    return true;
                }
                if !has_fit
                    && fallback.is_none()
                    && label.nothing_fits == NothingFits::LargestProviderSize
                {
                    fallback = provider
                        .largest_for_start()
                        .filter(|size| {
                            size.start_bound
                                .is_some_and(|bound| plan.can_reserve(id, bound))
                        })
                        .map(|size| (id, size));
                }
            }
            Place::Provider(_) => {}
        }
    }

    if let Some((provider, size)) = fallback.filter(|_| !has_fit) {
        return start_provider(
            plan,
            job,
            label,
            provider,
            size,
            StartReason::LargestProviderSize,
        );
    }
    false
}

fn start_provider(
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
    provider: ProviderId,
    size: &ProviderSize,
    reason: StartReason,
) -> bool {
    if !size
        .start_bound
        .is_some_and(|bound| plan.reserve_provider(provider, bound))
    {
        return false;
    }
    plan.start(
        job,
        label,
        size.size,
        Destination::Provider {
            provider,
            size_id: size.id.clone(),
        },
        reason,
    );
    true
}

pub(crate) fn starts_held(provider: &ProviderAccount, now: Duration) -> bool {
    provider.facts.refusals.iter().any(|refusal| match refusal {
        Refusal::RetryAfter { not_before } => now < *not_before,
        Refusal::RejectedKey | Refusal::BillingRequired => true,
        Refusal::SizeNotOnPlan(_) => false,
    })
}
