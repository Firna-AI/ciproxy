//! Boat availability, exact wait thresholds, and catalog size selection.

use ciproxy_protocol::resources::VmSize;

use crate::actions::{Destination, StartReason};
use crate::boat::{BoatAccount, BoatSize, Refusal};
use crate::capacity::eligible;
use crate::snapshot::{Label, NothingFits, Snapshot, WaitingJob};
use crate::state::Plan;

pub(crate) fn place_boat(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
) -> bool {
    if !label.destinations.boat || !available(&snapshot.boat, snapshot) {
        return false;
    }
    let own_fits = snapshot
        .machines
        .iter()
        .any(|machine| eligible(machine, label));
    let waited = snapshot.now.saturating_sub(job.queued_at);
    if own_fits && waited < snapshot.overflow_wait {
        return false;
    }
    let fitting = snapshot
        .boat
        .sizes
        .iter()
        .filter(|size| allowed(&snapshot.boat, size) && covers(size.size, label.size))
        .min_by_key(|size| size.price);
    let (size, reason) = if let Some(size) = fitting {
        let reason = if own_fits {
            StartReason::OverflowWaitElapsed { waited }
        } else {
            StartReason::NoEligibleMachine
        };
        (size, reason)
    } else {
        if own_fits || label.nothing_fits != NothingFits::LargestBoatSize {
            return false;
        }
        let Some(size) = snapshot.boat.sizes.iter().find(|size| {
            Some(&size.id) == snapshot.boat.largest_size.as_ref() && allowed(&snapshot.boat, size)
        }) else {
            return false;
        };
        (size, StartReason::LargestBoatSize)
    };
    plan.start(
        job,
        label,
        size.size,
        Destination::Boat(size.id.clone()),
        reason,
    );
    true
}

fn available(boat: &BoatAccount, snapshot: &Snapshot) -> bool {
    boat.enabled
        && boat.connected
        && boat.spend.spent < boat.spend.limit
        && !boat.refusals.iter().any(|refusal| match refusal {
            Refusal::RetryAfter { not_before } => snapshot.now < *not_before,
            Refusal::RejectedKey | Refusal::BillingRequired => true,
            Refusal::SizeNotOnPlan(_) => false,
        })
}

fn allowed(boat: &BoatAccount, size: &BoatSize) -> bool {
    !boat
        .refusals
        .iter()
        .any(|refusal| matches!(refusal, Refusal::SizeNotOnPlan(id) if *id == size.id))
}

fn covers(actual: VmSize, requested: VmSize) -> bool {
    actual.vcpu >= requested.vcpu
        && actual.memory_bytes >= requested.memory_bytes
        && actual.disk_bytes >= requested.disk_bytes
}
