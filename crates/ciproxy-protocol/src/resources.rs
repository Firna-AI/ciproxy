//! Pure slot and job disk arithmetic, independent of runner scheduling.

use serde::{Deserialize, Serialize};

/// The number of bytes in one decimal GB, used for the documented sizes.
pub const GIGABYTE: u64 = 1_000_000_000;

/// The resources requested by one runner's VM.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VmSize {
    /// Hardware threads given to the VM.
    pub vcpu: u32,
    /// Memory given to the VM, in bytes.
    pub memory_bytes: u64,
    /// Disk requested by the label, in bytes.
    pub disk_bytes: u64,
}

/// Whole slots, each containing two hardware threads and eight GB.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Slots(
    /// The whole number of slots.
    pub u64,
);

impl Slots {
    /// Compute capacity after reserving one quarter of CPU and memory.
    ///
    /// A machine without KVM has no slots. Disk does not enter this formula.
    pub fn for_machine(threads: u32, memory_bytes: u64, kvm: bool) -> Self {
        if !kvm {
            return Self(0);
        }
        let cpu_slots = u64::from(threads) * 3 / 8;
        let memory_slots = (u128::from(memory_bytes) * 3 / (32 * u128::from(GIGABYTE))) as u64;
        Self(cpu_slots.min(memory_slots))
    }

    /// Compute the larger of rounded-up CPU and memory slot requirements.
    pub fn for_vm(size: VmSize) -> Self {
        let cpu_slots = u64::from(size.vcpu).div_ceil(2);
        let memory_slots = size.memory_bytes.div_ceil(8 * GIGABYTE);
        Self(cpu_slots.max(memory_slots))
    }

    /// Check only slots. Use [`Resources::fits`] to check disk as well.
    pub fn fits(self, required: Self) -> bool {
        self >= required
    }
}

/// Slots and job disk available to VMs on one machine.
///
/// This value can represent total capacity or the resources not held by VMs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Resources {
    /// Whole CPU and memory slots.
    pub slots: Slots,
    /// Job disk in bytes, including space used by reclaimable caches.
    pub job_disk_bytes: u64,
}

impl Resources {
    /// Compute capacity after the CPU, memory, and disk host reserves.
    ///
    /// Pass the capacity of the disk used for images, VMs, and machine cache.
    /// Filesystem free bytes are a health measurement, not this capacity.
    pub fn for_machine(threads: u32, memory_bytes: u64, disk_bytes: u64, kvm: bool) -> Self {
        Self {
            slots: Slots::for_machine(threads, memory_bytes, kvm),
            job_disk_bytes: (u128::from(disk_bytes) * 3 / 4) as u64,
        }
    }

    /// Check whether both slots and job disk cover the VM size.
    pub fn fits(self, size: VmSize) -> bool {
        self.concurrent_vms(size) > 0
    }

    /// Compute the whole VMs of one size that these resources can hold.
    ///
    /// A dimension requested as zero does not limit the count.
    pub fn concurrent_vms(self, size: VmSize) -> u64 {
        let by_slots = self
            .slots
            .0
            .checked_div(Slots::for_vm(size).0)
            .unwrap_or(u64::MAX);
        let by_disk = self
            .job_disk_bytes
            .checked_div(size.disk_bytes)
            .unwrap_or(u64::MAX);
        by_slots.min(by_disk)
    }

    /// Subtract the full sizes of VMs still held by the machine.
    ///
    /// Include starting, warm, running, and stopping VMs once each.
    /// Reclaimable machine and unused Docker caches do not hold resources.
    /// Occupancy beyond capacity leaves zero free resources in that dimension.
    pub fn available(self, held_vms: impl IntoIterator<Item = VmSize>) -> Self {
        held_vms.into_iter().fold(self, |free, size| Self {
            slots: Slots(free.slots.0.saturating_sub(Slots::for_vm(size).0)),
            job_disk_bytes: free.job_disk_bytes.saturating_sub(size.disk_bytes),
        })
    }
}

#[cfg(test)]
#[path = "_tests_/resources_tests.rs"]
mod resources_tests;

#[cfg(test)]
#[path = "_tests_/disk_resources_tests.rs"]
mod disk_resources_tests;
