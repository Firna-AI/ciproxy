//! CPU and memory slot arithmetic, independent of scheduling and disk policy.

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

    /// Check whether one machine has enough slots for a VM.
    pub fn fits(self, required: Self) -> bool {
        self >= required
    }
}

#[cfg(test)]
#[path = "_tests_/resources_tests.rs"]
mod resources_tests;
