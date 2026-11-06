//! Report mappings retain hardware and cache facts while marking stale values.

use chrono::{DateTime, Utc};
use ciproxy_gateway::types::{MachineView, Presence};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::resources::{Resources, VmSize};
use ciproxy_store_interface::metering::history::MachineCost;

use crate::capacity::Capacity;
use crate::wire::machines;
use crate::wire::money::Cents;

pub(super) fn machine(
    value: MachineView,
    costs: &[MachineCost],
    at: DateTime<Utc>,
    default_size: Option<VmSize>,
    capacity: Capacity,
) -> machines::MachineView {
    let machine = value.snapshot.machine;
    let resources = capacity.job_disk_bytes.map(|job_disk_bytes| Resources {
        slots: capacity.slots,
        job_disk_bytes,
    });
    let cost = costs
        .iter()
        .filter(|cost| {
            cost.machine_id == machine.id
                && cost.starts_at <= at
                && cost.ends_at.is_none_or(|end| at < end)
        })
        .max_by_key(|cost| cost.starts_at)
        .and_then(|cost| Cents::from_usd(cost.monthly_usd));
    let report = value.snapshot.report;
    let occupancy = report
        .as_ref()
        .map(|report| {
            report
                .slots
                .iter()
                .map(|slot| {
                    let (state, runner_id, job_id) = match &slot.occupancy {
                        SlotOccupancy::Free => (machines::SlotState::Free, None, None),
                        SlotOccupancy::Warm { runner_id } => {
                            (machines::SlotState::Warm, Some(runner_id.0.clone()), None)
                        }
                        SlotOccupancy::Job { runner_id, job_id } => (
                            machines::SlotState::Job,
                            Some(runner_id.0.clone()),
                            Some(job_id.0.clone()),
                        ),
                    };
                    machines::SlotView {
                        index: slot.index.to_string(),
                        state,
                        runner_id,
                        job_id,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let cache = report.map(|report| machines::CacheView {
        used_bytes: report.cache.used_bytes.to_string(),
        capacity_bytes: report.cache.capacity_bytes.to_string(),
        restores_last_7_days: report.cache.restores_last_7_days.to_string(),
        hits_last_7_days: report.cache.hits_last_7_days.to_string(),
        docker_disk_bytes: report.docker_disks.used_bytes.to_string(),
    });
    machines::MachineView {
        id: machine.id.to_string(),
        hostname: machine.hostname,
        processor: machine.processor,
        threads: machine.hardware_threads,
        memory_bytes: machine.memory_bytes.to_string(),
        kvm: machine.kvm_available,
        slots: machine.slots,
        selected_image_disk_bytes: capacity.image_disk_bytes.map(|bytes| bytes.to_string()),
        concurrent_default_jobs: resources
            .zip(default_size)
            .map(|(resources, size)| resources.concurrent_vms(size).to_string()),
        capacity_known: resources.is_some() && default_size.is_some(),
        offline: value.presence == Presence::Offline,
        health_old: value.presence == Presence::Offline,
        draining: machine.draining,
        warm_runners: machine.warm_runners,
        last_contact_at: machine.last_contact_at,
        agent_version: machine.agent_version,
        disks: machine
            .disks
            .into_iter()
            .map(|disk| machines::DiskView {
                path: disk.path,
                capacity_bytes: disk.total_bytes.to_string(),
                free_bytes: disk.free_bytes.to_string(),
            })
            .collect(),
        occupancy,
        cache,
        monthly_cost_cents: cost,
    }
}
