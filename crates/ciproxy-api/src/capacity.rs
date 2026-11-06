//! Trusted prepared machine capacity supplied by the hosted runtime.

use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::gateway::MachineSnapshot;
use unimock::unimock;

/// Actual prepared capacity, without a reconstructed default image budget.
#[derive(Clone, Copy, Debug)]
pub struct Capacity {
    /// CPU and memory slots reported by the agent.
    pub slots: Slots,
    /// Actual prepared job-area bytes, absent when the machine has not reported them.
    pub job_disk_bytes: Option<u64>,
    /// Full disk size of this machine's selected image, absent when unknown.
    pub image_disk_bytes: Option<u64>,
}

/// Runtime adapter for capacity from the exact same machine snapshot.
#[unimock(api = MachineCapacityMock)]
pub trait MachineCapacity: Send + Sync {
    /// Return actual capacity or explicit facts that are not yet available.
    fn capacity(&self, snapshot: &MachineSnapshot) -> Capacity;
    /// Return the actual required root allocation under the runtime's sizing policy.
    /// None preserves unavailable selected-image or sizing facts.
    fn required_root_bytes(
        &self,
        snapshot: &MachineSnapshot,
        requested_free_disk_bytes: u64,
    ) -> Option<u64>;
}
