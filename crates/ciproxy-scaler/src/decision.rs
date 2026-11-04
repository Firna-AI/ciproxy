//! Public pure entry point for a complete workspace decision.

use crate::actions::Action;
use crate::lifecycle::{claim_existing, retire_obsolete, retire_unneeded};
use crate::overflow::place_boat;
use crate::placement::{place_own, reserve_after_retirement};
use crate::snapshot::Snapshot;
use crate::state::Plan;
use crate::warm::replenish;

/// Return typed intents without mutating the input or performing IO.
///
/// Persist accepted starts before the next call. Retirement does not release
/// capacity until the runner is removed from a later confirmed snapshot.
pub fn decide(snapshot: &Snapshot) -> Vec<Action> {
    if !snapshot.control_plane_available {
        return Vec::new();
    }
    let mut plan = Plan::new(snapshot);
    retire_obsolete(snapshot, &mut plan);
    let mut uncovered = Vec::new();
    for job in &snapshot.queue {
        if snapshot.organizations.contains(&job.organization_id)
            && snapshot.labels.iter().any(|label| label.name == job.label)
            && !claim_existing(snapshot, &mut plan, job)
        {
            uncovered.push(job);
        }
    }
    let mut pending = Vec::new();
    for job in uncovered {
        let Some(label) = snapshot.labels.iter().find(|label| label.name == job.label) else {
            continue;
        };
        if !place_own(snapshot, &mut plan, job, label)
            && !place_boat(snapshot, &mut plan, job, label)
        {
            reserve_after_retirement(snapshot, &mut plan, label);
            pending.push(job);
        }
    }
    retire_unneeded(snapshot, &mut plan);
    replenish(snapshot, &mut plan, &pending);
    plan.actions
}
