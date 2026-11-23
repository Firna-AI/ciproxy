//! Trusted prepared machine capacity supplied by the hosted runtime.

use ciproxy_agent::root_sizing::RootFit;
use ciproxy_protocol::platform::Platform;
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
    /// Return the root that one VM holds under the runtime's sizing policy:
    /// its bytes, a known refusal such as a macOS label above the image's
    /// free bytes, or unknown when selected-image or sizing facts are absent.
    fn required_root(&self, snapshot: &MachineSnapshot, requested_free_disk_bytes: u64) -> RootFit;
}

/// Limit a count of concurrent job VMs to the VM limit of the platform.
///
/// A Mac runs at most 2 job VMs, whatever its slots and job disk hold.
pub(crate) fn platform_vms(platform: Platform, count: u64) -> u64 {
    platform.vm_limit().map_or(count, |limit| count.min(limit))
}
