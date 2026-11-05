//! Reconcile the caller's recent-job allocation under one machine warm cap.

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::capacity::eligible;
use crate::snapshot::{RunnerState, Snapshot, WaitingJob};
use crate::state::{Plan, warm_candidate};

pub(crate) fn replenish(snapshot: &Snapshot, plan: &mut Plan, pending: &[&WaitingJob]) {
    let Some(label) = snapshot
        .labels
        .iter()
        .find(|label| label.name.0 == "ciproxy")
    else {
        return;
    };
    let Some(size) = snapshot.root_sizing.vm_size(label.size) else {
        return;
    };
    for (index, machine) in snapshot.machines.iter().enumerate() {
        if !eligible(machine, label, snapshot.root_sizing) {
            continue;
        }
        let mut vacancies: Vec<_> = machine
            .warm_allocation
            .iter()
            .take(machine.warm_count)
            .filter(|organization| snapshot.organizations.contains(organization))
            .cloned()
            .collect();
        let mut occupied = 0;
        for runner in &machine.runners {
            if runner.label.0 == "ciproxy"
                && matches!(
                    runner.state,
                    RunnerState::Starting | RunnerState::Ready | RunnerState::Retiring
                )
            {
                occupied += 1;
            }
            if !warm_candidate(runner) || plan.retiring.contains(&runner.id) {
                continue;
            }
            if let Some(position) = vacancies
                .iter()
                .position(|id| *id == runner.organization_id)
            {
                vacancies.remove(position);
            } else if !plan.claimed.contains(&runner.id) {
                plan.stop(runner, StopReason::WarmAllocationChanged);
            }
        }
        for action in &plan.actions {
            if let Action::StartRunner(start) = action
                && start.label == label.name
                && start.destination == Destination::Machine(machine.id.clone())
            {
                occupied += 1;
                if let Some(position) = vacancies.iter().position(|id| *id == start.organization_id)
                {
                    vacancies.remove(position);
                }
            }
        }
        if pending.iter().any(|job| {
            snapshot
                .labels
                .iter()
                .find(|candidate| candidate.name == job.label)
                .is_some_and(|candidate| eligible(machine, candidate, snapshot.root_sizing))
        }) {
            continue;
        }
        for organization_id in vacancies
            .into_iter()
            .take(machine.warm_count.saturating_sub(occupied))
        {
            if !plan.usage[index].fits(machine, size) {
                break;
            }
            plan.usage[index].add(size);
            let demand = WaitingJob {
                organization_id,
                label: label.name.clone(),
                queued_at: snapshot.now,
            };
            plan.start(
                &demand,
                label,
                size,
                Destination::Machine(machine.id.clone()),
                StartReason::WarmPool,
            );
        }
    }
}
