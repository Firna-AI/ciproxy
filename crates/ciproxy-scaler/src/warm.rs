//! Apply existing warm targets before checking capacity for new warm starts.

use ciproxy_protocol::identifiers::OrganizationId;

use crate::actions::{Action, Destination, StartReason, StopReason};
use crate::capacity::eligible;
use crate::snapshot::{Machine, RunnerState, Snapshot, WaitingJob};
use crate::state::{Plan, warm_candidate};

pub(crate) fn reconcile(snapshot: &Snapshot, plan: &mut Plan) {
    for machine in snapshot
        .machines
        .iter()
        .filter(|machine| machine.connected && !machine.draining)
    {
        let mut vacancies = targets(snapshot, machine);
        for runner in &machine.runners {
            if !warm_candidate(runner) || plan.retiring.contains(&runner.id) {
                continue;
            }
            if !take_target(&mut vacancies, &runner.organization_id)
                && !plan.claimed.contains(&runner.id)
            {
                plan.stop(runner, StopReason::WarmAllocationChanged);
            }
        }
    }
}

pub(crate) fn replenish(snapshot: &Snapshot, plan: &mut Plan, pending: &[&WaitingJob]) {
    let Some(label) = snapshot
        .labels
        .iter()
        .find(|label| label.name.0 == "ciproxy")
    else {
        return;
    };
    for (index, machine) in snapshot.machines.iter().enumerate() {
        if !eligible(machine, label, snapshot.root_sizing) {
            continue;
        }
        let Some(size) = machine
            .root_sizing(snapshot.root_sizing)
            .vm_size(label.size)
        else {
            continue;
        };
        let mut vacancies = targets(snapshot, machine);
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
            if warm_candidate(runner) && !plan.retiring.contains(&runner.id) {
                take_target(&mut vacancies, &runner.organization_id);
            }
        }
        for action in &plan.actions {
            if let Action::StartRunner(start) = action
                && start.label == label.name
                && start.destination == Destination::Machine(machine.id.clone())
            {
                occupied += 1;
                take_target(&mut vacancies, &start.organization_id);
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

fn targets(snapshot: &Snapshot, machine: &Machine) -> Vec<OrganizationId> {
    machine
        .warm_allocation
        .iter()
        .take(machine.warm_count)
        .filter(|organization| snapshot.organizations.contains(organization))
        .cloned()
        .collect()
}

fn take_target(vacancies: &mut Vec<OrganizationId>, organization: &OrganizationId) -> bool {
    let Some(position) = vacancies.iter().position(|id| id == organization) else {
        return false;
    };
    vacancies.remove(position);
    true
}
