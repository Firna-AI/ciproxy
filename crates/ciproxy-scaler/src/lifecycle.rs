//! Retire obsolete idle registrations while preserving every running job.

use crate::actions::StopReason;
use crate::snapshot::{Runner, RunnerState, Snapshot, WaitingJob};
use crate::state::{Plan, idle, label_for, warm_candidate};

pub(crate) fn retire_obsolete(snapshot: &Snapshot, plan: &mut Plan) {
    for machine in &snapshot.machines {
        for runner in &machine.runners {
            let reason = obsolete(snapshot, runner, true).or_else(|| {
                (machine.draining && idle(runner)).then_some(StopReason::MachineDraining)
            });
            if let Some(reason) = reason {
                plan.stop(runner, reason);
            }
        }
    }
    for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
        let reason = match &sandbox.runner {
            Some(runner) => obsolete(snapshot, runner, false),
            None => Some(StopReason::OrphanSandbox),
        };
        if let Some(reason) = reason {
            plan.delete(sandbox, reason);
        }
    }
}

fn obsolete(snapshot: &Snapshot, runner: &Runner, own_machine: bool) -> Option<StopReason> {
    if runner.state == RunnerState::Exited {
        return Some(StopReason::RunnerExited);
    }
    if !idle(runner) {
        return None;
    }
    if !snapshot.organizations.contains(&runner.organization_id) {
        return Some(StopReason::OrganizationRemoved);
    }
    let Some(label) = label_for(snapshot, runner) else {
        return Some(StopReason::LabelChanged);
    };
    if label.size != runner.requested_size {
        return Some(StopReason::LabelChanged);
    }
    if (own_machine && !label.destinations.own_machines)
        || (!own_machine && !label.destinations.boat)
    {
        return Some(StopReason::DestinationDisabled);
    }
    None
}

pub(crate) fn claim_existing(snapshot: &Snapshot, plan: &mut Plan, job: &WaitingJob) -> bool {
    for machine in &snapshot.machines {
        for runner in &machine.runners {
            if (machine.connected || runner.state == RunnerState::Ready) && claim(plan, runner, job)
            {
                return true;
            }
        }
    }
    for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
        if let Some(runner) = &sandbox.runner
            && claim(plan, runner, job)
        {
            return true;
        }
    }
    false
}

fn claim(plan: &mut Plan, runner: &Runner, job: &WaitingJob) -> bool {
    idle(runner)
        && runner.organization_id == job.organization_id
        && runner.label == job.label
        && !plan.retiring.contains(&runner.id)
        && plan.claimed.insert(runner.id.clone())
}

pub(crate) fn retire_unneeded(snapshot: &Snapshot, plan: &mut Plan) {
    for machine in &snapshot.machines {
        for runner in &machine.runners {
            if idle(runner) && !warm_candidate(runner) && !plan.claimed.contains(&runner.id) {
                plan.stop(runner, StopReason::NoWaitingJob);
            }
        }
    }
    for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
        if let Some(runner) = &sandbox.runner
            && idle(runner)
            && !plan.claimed.contains(&runner.id)
        {
            plan.delete(sandbox, StopReason::NoWaitingJob);
        }
    }
}
