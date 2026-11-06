//! Whole-GB label requests, distinct from byte-precise actual VM allocation.

use crate::error::{Error, Result, ResultExt};
use serde::{Deserialize, Serialize};

/// The resources of one job VM or Boat sandbox.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct VmSize {
    /// The virtual CPU count.
    pub vcpus: i32,
    /// Memory in GB.
    pub memory_gb: i32,
    /// Disk capacity in GB.
    pub disk_gb: i32,
}

impl VmSize {
    /// Express this stored whole-GB request in the public wire's byte units.
    pub fn bytes(self) -> Result<ciproxy_protocol::resources::VmSize> {
        Ok(ciproxy_protocol::resources::VmSize {
            vcpu: u32::try_from(self.vcpus).internal()?,
            memory_bytes: u64::try_from(self.memory_gb)
                .internal()?
                .saturating_mul(1_000_000_000),
            disk_bytes: u64::try_from(self.disk_gb)
                .internal()?
                .saturating_mul(1_000_000_000),
        })
    }
    /// Read an original whole-GB request from an authenticated agent report.
    pub fn from_request(size: ciproxy_protocol::resources::VmSize) -> Result<Self> {
        if !size.memory_bytes.is_multiple_of(1_000_000_000)
            || !size.disk_bytes.is_multiple_of(1_000_000_000)
        {
            return Err(Error::InvalidValue);
        }
        Ok(Self {
            vcpus: i32::try_from(size.vcpu).internal()?,
            memory_gb: i32::try_from(size.memory_bytes / 1_000_000_000).internal()?,
            disk_gb: i32::try_from(size.disk_bytes / 1_000_000_000).internal()?,
        })
    }
}
