//! Resource sizes kept with labels and runner records.

/// The resources of one job VM or Boat sandbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VmSize {
    /// The virtual CPU count.
    pub vcpus: i32,
    /// Memory in GB.
    pub memory_gb: i32,
    /// Disk capacity in GB.
    pub disk_gb: i32,
}
