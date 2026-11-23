//! CPU, memory, and job-disk reservations on one own machine.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::resources::{Resources, Slots, VmSize};

use crate::snapshot::{Label, Machine};

#[derive(Clone, Copy, Default)]
pub(crate) struct Usage {
    slots: u64,
    disk_bytes: u64,
    vms: u64,
    /// The disk of the VMs that this decision starts. The real free space
    /// of a Mac already counts the space that its running VMs took.
    started_disk_bytes: u64,
}

impl Usage {
    pub(crate) fn for_machine(machine: &Machine) -> Self {
        let mut result = Self::default();
        for runner in &machine.runners {
            result.hold(runner.size);
        }
        result
    }

    /// Hold the disk and slots of a VM that this decision starts.
    pub(crate) fn add(&mut self, size: VmSize) {
        self.hold(size);
        self.started_disk_bytes = self.started_disk_bytes.saturating_add(size.disk_bytes);
    }

    fn hold(&mut self, size: VmSize) {
        self.slots = self.slots.saturating_add(Slots::for_vm(size).0);
        self.disk_bytes = self.disk_bytes.saturating_add(size.disk_bytes);
        self.vms = self.vms.saturating_add(1);
    }

    pub(crate) fn remove(&mut self, size: VmSize) {
        self.slots = self.slots.saturating_sub(Slots::for_vm(size).0);
        self.disk_bytes = self.disk_bytes.saturating_sub(size.disk_bytes);
        self.vms = self.vms.saturating_sub(1);
    }

    pub(crate) fn add_usage(&mut self, other: Self) {
        self.slots = self.slots.saturating_add(other.slots);
        self.disk_bytes = self.disk_bytes.saturating_add(other.disk_bytes);
        self.vms = self.vms.saturating_add(other.vms);
        self.started_disk_bytes = self
            .started_disk_bytes
            .saturating_add(other.started_disk_bytes);
    }

    /// Whether a VM of `size` can start now: it fits what the machine holds
    /// and the real free space of its last report.
    pub(crate) fn fits(self, machine: &Machine, size: VmSize) -> bool {
        // The agent refuses a new VM that the real free space cannot hold,
        // so placement picks a machine whose last report has that space.
        let real_space = machine
            .free_bytes
            .is_none_or(|free| free >= self.started_disk_bytes.saturating_add(size.disk_bytes));
        real_space && self.fits_held(machine, size)
    }

    /// Whether a VM of `size` fits the slots, the VM limit, and the job disk
    /// that the machine holds. A retirement projection uses this: a stop
    /// frees real space that only the next report measures, and the start
    /// that follows checks it then.
    pub(crate) fn fits_held(self, machine: &Machine, size: VmSize) -> bool {
        let Some(leased_bytes) = machine.docker_leased_bytes else {
            return false;
        };
        if machine
            .platform
            .vm_limit()
            .is_some_and(|limit| self.vms >= limit)
        {
            return false;
        }
        Resources {
            slots: Slots(machine.capacity.0.saturating_sub(self.slots)),
            job_disk_bytes: machine
                .disk_budget_bytes
                .saturating_sub(leased_bytes)
                .saturating_sub(self.disk_bytes),
        }
        .fits(size)
    }
}

pub(crate) fn eligible(machine: &Machine, label: &Label, sizing: RootSizing) -> bool {
    let capacity = Resources {
        slots: machine.capacity,
        job_disk_bytes: machine
            .docker_leased_bytes
            .map_or(machine.disk_budget_bytes, |leased| {
                machine.disk_budget_bytes.saturating_sub(leased)
            }),
    };
    label.destinations.own_machines
        && machine.platform == label.platform
        && machine.platform.vm_limit() != Some(0)
        && machine.connected
        && !machine.draining
        && machine
            .root_rule(sizing)
            .vm_size(label.size)
            .is_some_and(|size| capacity.fits(size))
}
