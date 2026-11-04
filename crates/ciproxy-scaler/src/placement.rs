//! Own-machine placement and separate projected reservations for idle stops.

use crate::actions::{Destination, StartReason, StopReason};
use crate::capacity::{Usage, eligible};
use crate::snapshot::{Label, RunnerState, Snapshot, WaitingJob};
use crate::state::{Plan, warm_candidate};

pub(crate) fn place_own(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    label: &Label,
) -> bool {
    for (index, machine) in snapshot.machines.iter().enumerate() {
        if eligible(machine, label)
            && plan.usage[index].fits(machine, label.size)
            && projected_usage(snapshot, plan, index).fits(machine, label.size)
        {
            plan.usage[index].add(label.size);
            plan.start(
                job,
                label,
                label.size,
                Destination::Machine(machine.id.clone()),
                StartReason::OwnMachineCapacity,
            );
            return true;
        }
    }
    false
}

pub(crate) fn reserve_after_retirement(snapshot: &Snapshot, plan: &mut Plan, label: &Label) {
    for (index, machine) in snapshot.machines.iter().enumerate() {
        if !eligible(machine, label) {
            continue;
        }
        let mut future = projected_usage(snapshot, plan, index);
        let mut retire = Vec::new();
        for runner in &machine.runners {
            if future.fits(machine, label.size) {
                break;
            }
            if warm_candidate(runner)
                && !plan.claimed.contains(&runner.id)
                && !plan.retiring.contains(&runner.id)
            {
                future.remove(runner.size);
                retire.push(runner);
            }
        }
        if future.fits(machine, label.size) {
            for runner in retire {
                plan.stop(runner, StopReason::WarmPreemption);
            }
            plan.reserved[index].add(label.size);
            return;
        }
    }
}

fn projected_usage(snapshot: &Snapshot, plan: &Plan, index: usize) -> Usage {
    let mut future = plan.usage[index];
    for runner in &snapshot.machines[index].runners {
        if plan.retiring.contains(&runner.id) || runner.state == RunnerState::Retiring {
            future.remove(runner.size);
        }
    }
    future.add_usage(plan.reserved[index]);
    future
}
