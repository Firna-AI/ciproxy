//! Report mappings retain hardware and cache facts while marking stale values.

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootFit;
use ciproxy_gateway::types::{MachineView, Presence};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::resources::{Resources, Slots, VmSize};
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::metering::history::MachineCost;

use crate::capacity::{Capacity, platform_vms};
use crate::wire::machine_statistics::{CacheView, DockerDiskView, MachineCacheView};
use crate::wire::machines;
use crate::wire::money::Cents;

use super::machine_jobs;

/// Default request and machine-selected allocation remain separate facts.
pub(super) struct DefaultSize {
    /// Saved default-label CPU, memory, and free-disk request.
    pub requested: VmSize,
    /// Runtime root under the machine-selected image policy.
    pub root: RootFit,
}

pub(super) fn machine(
    value: MachineView,
    costs: &[MachineCost],
    at: DateTime<Utc>,
    default_size: Option<DefaultSize>,
    capacity: Capacity,
    jobs: &[DashboardJob],
    latest_agent_version: Option<&str>,
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
    let report = value.snapshot.report.map(|received| received.report);
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
                        job: machine_jobs::resolve(jobs, machine.id, slot, at),
                        index: slot.index.to_string(),
                        state,
                        runner_id,
                        job_id,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let cache = report.map(|report| CacheView {
        machine_cache: report.cache.map(|cache| MachineCacheView {
            used_bytes: cache.used_bytes.to_string(),
            capacity_bytes: cache.capacity_bytes.to_string(),
            restores_last_7_days: cache.restores_last_7_days.to_string(),
            hits_last_7_days: cache.hits_last_7_days.to_string(),
        }),
        docker_disks: report.docker_disks.map(|docker| DockerDiskView {
            used_bytes: docker.used_bytes.to_string(),
            repositories: docker.repositories.to_string(),
        }),
    });
    let by_slots = default_size.as_ref().and_then(|size| {
        capacity
            .slots
            .0
            .checked_div(Slots::for_vm(size.requested).0)
            .map(|count| platform_vms(machine.platform, count))
    });
    let root = default_size.as_ref().and_then(|size| match size.root {
        RootFit::Holds(bytes) => Some(bytes),
        RootFit::AboveImage | RootFit::Unknown => None,
    });
    // The default label asks for more free disk than the image has: the
    // machine runs none of its jobs, and more host disk would not help.
    let refused = default_size
        .as_ref()
        .is_some_and(|size| size.root == RootFit::AboveImage);
    let allocated = default_size.as_ref().and_then(|size| {
        root.map(|disk_bytes| VmSize {
            disk_bytes,
            ..size.requested
        })
    });
    let concurrent = if refused {
        Some(0)
    } else {
        resources
            .zip(allocated)
            .map(|(resources, size)| platform_vms(machine.platform, resources.concurrent_vms(size)))
    };
    machines::MachineView {
        default_jobs_by_slots: by_slots.map(|count| count.to_string()),
        default_root_disk_bytes: root.map(|bytes| bytes.to_string()),
        job_disk_bytes: capacity.job_disk_bytes.map(|bytes| bytes.to_string()),
        disk_limited: if refused {
            Some(false)
        } else {
            concurrent
                .zip(by_slots)
                .map(|(concurrent, slots)| concurrent < slots)
        },
        agent_up_to_date: latest_agent_version.map(|latest| latest == machine.agent_version),
        latest_agent_version: latest_agent_version.map(str::to_owned),
        id: machine.id.to_string(),
        hostname: machine.hostname,
        processor: machine.processor,
        threads: machine.hardware_threads,
        memory_bytes: machine.memory_bytes.to_string(),
        platform: machine.platform,
        virtualization: machine.virtualization,
        kvm: machine.virtualization.is_available(),
        vm_limit: machine
            .platform
            .vm_limit()
            .map(|limit| u32::try_from(limit).unwrap_or(u32::MAX)),
        slots: machine.slots,
        selected_image_disk_bytes: capacity.image_disk_bytes.map(|bytes| bytes.to_string()),
        concurrent_default_jobs: concurrent.map(|count| count.to_string()),
        capacity_known: resources.is_some() && allocated.is_some(),
        state: value.state.into(),
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
