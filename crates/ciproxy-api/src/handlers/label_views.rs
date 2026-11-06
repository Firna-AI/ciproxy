//! Label resources and connected-machine fit from the shared capacity arithmetic.

use ciproxy_gateway::types::{MachineView, Presence};
use ciproxy_protocol::resources::{GIGABYTE, Resources, VmSize};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use uuid::Uuid;

use crate::capacity::MachineCapacity;
use crate::error::{Error, Result};
use crate::wire::settings::{LabelFit, LabelInput, LabelView, NothingFits};

pub(super) fn stored(id: Uuid, input: LabelInput) -> Result<Label> {
    if input.vcpus < 1 || input.memory_gb < 1 || input.disk_gb < 1 {
        return Err(Error::InvalidInput);
    }
    Ok(Label {
        id,
        name: input.name,
        size: ciproxy_store_interface::vm_size::VmSize {
            vcpus: input.vcpus,
            memory_gb: input.memory_gb,
            disk_gb: input.disk_gb,
        },
        allow_machines: input.allow_machines,
        allow_boat: input.allow_boat,
        when_nothing_fits: match input.when_nothing_fits {
            NothingFits::Wait => WhenNothingFits::Wait,
            NothingFits::LargestBoat => WhenNothingFits::LargestBoat,
        },
    })
}

pub(super) fn label(
    label: Label,
    machines: &[MachineView],
    capacity: &dyn MachineCapacity,
) -> LabelView {
    let input = LabelInput {
        name: label.name,
        vcpus: label.size.vcpus,
        memory_gb: label.size.memory_gb,
        disk_gb: label.size.disk_gb,
        allow_machines: label.allow_machines,
        allow_boat: label.allow_boat,
        when_nothing_fits: match label.when_nothing_fits {
            WhenNothingFits::Wait => NothingFits::Wait,
            WhenNothingFits::LargestBoat => NothingFits::LargestBoat,
        },
    };
    let fit = fit(&input, machines, capacity);
    LabelView {
        id: label.id.to_string(),
        label: input,
        fit,
    }
}

pub(super) fn fit(
    label: &LabelInput,
    machines: &[MachineView],
    capacity: &dyn MachineCapacity,
) -> LabelFit {
    let size = VmSize {
        vcpu: label.vcpus.try_into().unwrap_or(0),
        memory_bytes: u64::try_from(label.memory_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
        disk_bytes: u64::try_from(label.disk_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
    };
    let mut counts = Vec::new();
    let eligible = machines
        .iter()
        .filter(|machine| {
            machine.presence == Presence::Online
                && !machine.snapshot.machine.draining
                && machine.snapshot.machine.kvm_available
                && machine.snapshot.machine.slots > 0
        })
        .collect::<Vec<_>>();
    let mut unavailable = 0u32;
    for view in &eligible {
        let facts = capacity.capacity(&view.snapshot);
        let root_bytes = capacity.required_root_bytes(&view.snapshot, size.disk_bytes);
        let (Some(job_disk_bytes), Some(root_bytes)) = (facts.job_disk_bytes, root_bytes) else {
            unavailable += 1;
            continue;
        };
        let resources = Resources {
            slots: facts.slots,
            job_disk_bytes,
        };
        let jobs = if label.allow_machines {
            resources.concurrent_vms(VmSize {
                disk_bytes: root_bytes,
                ..size
            })
        } else {
            0
        };
        if jobs > 0 {
            counts.push(jobs);
        }
    }
    LabelFit {
        unavailable_machines: unavailable,
        eligible_machines: eligible.len().try_into().unwrap_or(u32::MAX),
        fitting_machines: counts.len().try_into().unwrap_or(u32::MAX),
        minimum_jobs: counts.iter().min().map(ToString::to_string),
        maximum_jobs: counts.iter().max().map(ToString::to_string),
        no_machine_fits: counts.is_empty() && unavailable == 0,
    }
}
