//! Boat availability, exact wait thresholds, and catalog size selection.

use crate::actions::{Destination, StartReason};
use crate::boat::{BoatAccount, Refusal};
use crate::size_policy::{BoatChoice, SizePolicy};
use crate::snapshot::{Label, Snapshot, WaitingJob};
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
    let sizes = SizePolicy::for_label(snapshot, label);
    let waited = snapshot.now.saturating_sub(job.queued_at);
    if sizes.own_fits && waited < snapshot.overflow_wait {
        return false;
    }
    let (size, reason) = match sizes.boat {
        Some(BoatChoice::Fitting(size)) => {
            let reason = if sizes.own_fits {
                StartReason::OverflowWaitElapsed { waited }
            } else {
                StartReason::NoEligibleMachine
            };
            (size, reason)
        }
        Some(BoatChoice::Largest(size)) => (size, StartReason::LargestBoatSize),
        None => return false,
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
