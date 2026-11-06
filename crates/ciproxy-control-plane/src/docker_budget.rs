//! Conservative spare capacity never understates a feasible root allocation.

use std::collections::BTreeMap;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::cache::DockerBudget;
use ciproxy_protocol::resources::{Slots, VmSize};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

use crate::snapshot::size;

pub(crate) fn policy(input: &RoutingSnapshot, machine_id: Uuid) -> Option<DockerBudget> {
    let report = input
        .machines
        .iter()
        .find(|machine| machine.machine.id == machine_id)?
        .report
        .as_ref()?;
    let image = report.images.selected()?.clone();
    let sizing = RootSizing::for_image(&image);
    let roots: Vec<_> = input
        .labels
        .iter()
        .filter(|label| label.allow_machines)
        .filter_map(|label| sizing.vm_size(size(label.size)))
        .map(|size| (Slots::for_vm(size).0, size.disk_bytes))
        .collect();
    let mut held:BTreeMap<String,VmSize> = input.runners.iter()
        .filter(|runner| runner.runner.state != RunnerState::Stopped
            && matches!(runner.runner.place,RunnerPlace::Machine {machine_id:owner,..} if owner == machine_id))
        .map(|runner| (runner.runner.id.to_string(),runner.runner.size)).collect();
    for runner in &report.runners {
        held.entry(runner.runner_id.0.clone())
            .and_modify(|allocation| {
                allocation.vcpu = allocation.vcpu.max(runner.size.vcpu);
                allocation.memory_bytes = allocation.memory_bytes.max(runner.size.memory_bytes);
                allocation.disk_bytes = allocation.disk_bytes.max(runner.size.disk_bytes);
            })
            .or_insert(runner.size);
    }
    let retained: Vec<_> = held
        .values()
        .map(|size| (Slots::for_vm(*size).0, size.disk_bytes))
        .collect();
    Some(DockerBudget {
        image,
        spare_bytes: spare(report.capacity.0, report.job_disk_bytes, &roots, &retained),
    })
}

pub(crate) fn spare(slots: u64, job_disk: u64, roots: &[(u64, u64)], held: &[(u64, u64)]) -> u64 {
    let per_slot = roots
        .iter()
        .filter(|(width, bytes)| *width > 0 && *width <= slots && *bytes <= job_disk)
        .map(|(width, bytes)| u128::from(*bytes).div_ceil(u128::from(*width)))
        .max()
        .unwrap_or(0);
    let excess: u128 = held
        .iter()
        .map(|(width, bytes)| {
            u128::from(*bytes).saturating_sub(per_slot.saturating_mul(u128::from(*width)))
        })
        .sum();
    let occupied = per_slot
        .saturating_mul(u128::from(slots))
        .saturating_add(excess)
        .min(u128::from(job_disk));
    job_disk.saturating_sub(occupied as u64)
}

#[cfg(test)]
#[path = "_tests_/docker_budget_tests.rs"]
mod docker_budget_tests;
