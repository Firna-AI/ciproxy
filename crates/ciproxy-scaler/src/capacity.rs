//! CPU, memory, and caller-derived disk reservations on one own machine.

use ciproxy_protocol::resources::{Slots, VmSize};

use crate::snapshot::{Label, Machine};

#[derive(Clone, Copy, Default)]
pub(crate) struct Usage {
    slots: u64,
    disk_bytes: u64,
}

impl Usage {
    pub(crate) fn for_machine(machine: &Machine) -> Self {
        let mut result = Self {
            slots: 0,
            disk_bytes: 0,
        };
        for runner in &machine.runners {
            result.add(runner.size);
        }
        result
    }

    pub(crate) fn add(&mut self, size: VmSize) {
        self.slots = self.slots.saturating_add(Slots::for_vm(size).0);
        self.disk_bytes = self.disk_bytes.saturating_add(size.disk_bytes);
    }

    pub(crate) fn remove(&mut self, size: VmSize) {
        self.slots = self.slots.saturating_sub(Slots::for_vm(size).0);
        self.disk_bytes = self.disk_bytes.saturating_sub(size.disk_bytes);
    }

    pub(crate) fn add_usage(&mut self, other: Self) {
        self.slots = self.slots.saturating_add(other.slots);
        self.disk_bytes = self.disk_bytes.saturating_add(other.disk_bytes);
    }

    pub(crate) fn fits(self, machine: &Machine, size: VmSize) -> bool {
        machine.capacity.0.saturating_sub(self.slots) >= Slots::for_vm(size).0
            && machine.disk_budget_bytes.saturating_sub(self.disk_bytes) >= size.disk_bytes
    }
}

pub(crate) fn eligible(machine: &Machine, label: &Label) -> bool {
    label.destinations.own_machines
        && machine.connected
        && !machine.draining
        && machine.capacity.fits(Slots::for_vm(label.size))
        && machine.disk_budget_bytes >= label.size.disk_bytes
}
