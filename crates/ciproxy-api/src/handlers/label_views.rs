//! Label resources and connected-machine fit from the shared capacity arithmetic.

use ciproxy_gateway::types::{MachineView, Presence};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{GIGABYTE, Resources, VmSize};
use ciproxy_store_interface::labels::{self, Label, WhenNothingFits};
use uuid::Uuid;

use crate::capacity::{MachineCapacity, platform_vms};
use crate::error::{Error, Result};
use crate::wire::facts::LabelMachineFit;
use crate::wire::job_fit::{LabelFit, LabelRouting, LabelWaitReason};
use crate::wire::settings::{LabelInput, LabelPlace, LabelSpec, LabelView, NothingFits};

use super::{label_provider::Observation, label_routing};

pub(super) fn stored(id: Uuid, spec: LabelSpec, mut order: Vec<LabelPlace>) -> Result<Label> {
    if spec.vcpus < 1 || spec.memory_gb < 1 || spec.disk_gb < 1 {
        return Err(Error::InvalidInput);
    }
    if order
        .iter()
        .enumerate()
        .any(|(index, place)| order[..index].contains(place))
    {
        return Err(Error::InvalidInput);
    }
    if spec.platform == Platform::MacosArm64 {
        if order.iter().any(|place| *place != LabelPlace::Machine)
            || !matches!(spec.when_nothing_fits, NothingFits::Wait)
        {
            return Err(Error::InvalidInput);
        }
        order = vec![LabelPlace::Machine];
    }
    Ok(Label {
        id,
        name: spec.name,
        platform: spec.platform,
        size: ciproxy_store_interface::vm_size::VmSize {
            vcpus: spec.vcpus,
            memory_gb: spec.memory_gb,
            disk_gb: spec.disk_gb,
        },
        order: order
            .into_iter()
            .map(|place| match place {
                LabelPlace::Machine => labels::LabelPlace::Machine,
                LabelPlace::Overflow { provider } => labels::LabelPlace::Provider(provider.0),
            })
            .collect(),
        when_nothing_fits: match spec.when_nothing_fits {
            NothingFits::Wait => WhenNothingFits::Wait,
            NothingFits::LargestOverflow => WhenNothingFits::LargestOverflow,
        },
    })
}

pub(super) fn label(
    label: Label,
    machines: &[MachineView],
    capacity: &dyn MachineCapacity,
    observation: &Observation<'_>,
) -> LabelView {
    let input = LabelInput {
        spec: LabelSpec {
            name: label.name,
            platform: label.platform,
            vcpus: label.size.vcpus,
            memory_gb: label.size.memory_gb,
            disk_gb: label.size.disk_gb,
            when_nothing_fits: match label.when_nothing_fits {
                WhenNothingFits::Wait => NothingFits::Wait,
                WhenNothingFits::LargestOverflow => NothingFits::LargestOverflow,
            },
        },
        order: label
            .order
            .into_iter()
            .map(|place| match place {
                labels::LabelPlace::Machine => LabelPlace::Machine,
                labels::LabelPlace::Provider(provider) => LabelPlace::Overflow {
                    provider: provider.into(),
                },
            })
            .collect(),
    };
    let size = VmSize {
        vcpu: input.spec.vcpus.try_into().unwrap_or(0),
        memory_bytes: u64::try_from(input.spec.memory_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
        disk_bytes: u64::try_from(input.spec.disk_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
    };
    let mut fit = machine_fit(&input, size, machines, capacity);
    label_routing::enrich(&mut fit, &input, size, observation);
    LabelView {
        id: label.id.to_string(),
        label: input,
        fit,
        jobs_last_30_days: None,
        job_count_since: None,
        job_count_until: None,
    }
}

fn machine_fit(
    label: &LabelInput,
    size: VmSize,
    machines: &[MachineView],
    capacity: &dyn MachineCapacity,
) -> LabelFit {
    let allows_machines = label.order.contains(&LabelPlace::Machine);
    let mut counts = Vec::new();
    let mut offers = Vec::new();
    let eligible = machines
        .iter()
        .filter(|machine| {
            machine.presence == Presence::Online
                && !machine.snapshot.machine.draining
                && machine.snapshot.machine.platform == label.spec.platform
                && machine.snapshot.machine.virtualization.is_available()
                && machine.snapshot.machine.slots > 0
        })
        .collect::<Vec<_>>();
    let mut unavailable = 0u32;
    for view in &eligible {
        let facts = capacity.capacity(&view.snapshot);
        let root_bytes = capacity.required_root_bytes(&view.snapshot, size.disk_bytes);
        let concurrent_jobs =
            facts
                .job_disk_bytes
                .zip(root_bytes)
                .map(|(job_disk_bytes, root_bytes)| {
                    if allows_machines {
                        let jobs = Resources {
                            slots: facts.slots,
                            job_disk_bytes,
                        }
                        .concurrent_vms(VmSize {
                            disk_bytes: root_bytes,
                            ..size
                        });
                        platform_vms(label.spec.platform, jobs)
                    } else {
                        0
                    }
                });
        offers.push(LabelMachineFit {
            id: view.snapshot.machine.id.to_string(),
            hostname: view.snapshot.machine.hostname.clone(),
            slots: facts.slots.0.to_string(),
            vcpus: facts.slots.0.saturating_mul(2).to_string(),
            memory_bytes: facts.slots.0.saturating_mul(8 * GIGABYTE).to_string(),
            job_disk_bytes: facts.job_disk_bytes.map(|bytes| bytes.to_string()),
            required_root_disk_bytes: root_bytes.map(|bytes| bytes.to_string()),
            concurrent_jobs: concurrent_jobs.map(|jobs| jobs.to_string()),
        });
        let (Some(job_disk_bytes), Some(root_bytes)) = (facts.job_disk_bytes, root_bytes) else {
            unavailable += 1;
            continue;
        };
        let resources = Resources {
            slots: facts.slots,
            job_disk_bytes,
        };
        let jobs = if allows_machines {
            platform_vms(
                label.spec.platform,
                resources.concurrent_vms(VmSize {
                    disk_bytes: root_bytes,
                    ..size
                }),
            )
        } else {
            0
        };
        if jobs > 0 {
            counts.push(jobs);
        }
    }
    LabelFit {
        machines: offers,
        providers: Vec::new(),
        fitting_places: None,
        routing: LabelRouting::Waits {
            reason: LabelWaitReason::Unavailable,
        },
        unavailable_machines: unavailable,
        eligible_machines: eligible.len().try_into().unwrap_or(u32::MAX),
        fitting_machines: counts.len().try_into().unwrap_or(u32::MAX),
        minimum_jobs: counts.iter().min().map(ToString::to_string),
        maximum_jobs: counts.iter().max().map(ToString::to_string),
        no_machine_fits: counts.is_empty() && unavailable == 0,
    }
}
