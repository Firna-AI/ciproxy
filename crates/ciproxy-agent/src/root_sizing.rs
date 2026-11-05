//! One root allocation includes the image, free job bytes, and ext4 overhead.

use ciproxy_protocol::resources::{GIGABYTE, VmSize};

/// Provisional logical image length until the smallest full-image export is measured.
pub const DEFAULT_IMAGE_ROOT_BYTES: u64 = 61 * GIGABYTE;

/// The smallest free job disk accepted for a label.
pub const MINIMUM_JOB_DISK_BYTES: u64 = GIGABYTE;

/// Maximum kernel ext4 reserve pool: 4096 clusters of 4096 bytes each.
///
/// See `ext4_set_resv_clusters` and `ext4_statfs` in the Linux ext4 source:
/// <https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/333a396d71ce4083da31ccbf30772bf2da1509b4/fs/ext4/super.c>.
const EXT4_RESERVE_POOL_BYTES: u64 = 16 * 1024 * 1024;

/// Image geometry shared by the agent and each placement caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootSizing {
    /// Logical root disk bytes from the selected image release manifest.
    pub image_root_bytes: u64,
}

impl Default for RootSizing {
    fn default() -> Self {
        Self {
            image_root_bytes: DEFAULT_IMAGE_ROOT_BYTES,
        }
    }
}

impl RootSizing {
    /// Grow by free bytes plus a 1% root reserve and 2% new ext4 metadata.
    ///
    /// Images use 4096-byte blocks, 256-byte inodes, and at least 16384 bytes
    /// per inode, with no bigalloc clusters. The image contains its own
    /// metadata and root reserve. Include the kernel's separate reserve pool.
    /// Return `None` when the requested length cannot fit in a byte count.
    pub fn root_bytes(self, free_job_bytes: u64) -> Option<u64> {
        let growth =
            ((u128::from(free_job_bytes) + u128::from(EXT4_RESERVE_POOL_BYTES)) * 100).div_ceil(97);
        let bytes = u128::from(self.image_root_bytes) + growth;
        u64::try_from(bytes.div_ceil(4096) * 4096).ok()
    }

    /// Replace label free bytes with the complete VM root allocation.
    pub fn vm_size(self, label: VmSize) -> Option<VmSize> {
        Some(VmSize {
            disk_bytes: self.root_bytes(label.disk_bytes)?,
            ..label
        })
    }
}

#[cfg(test)]
#[path = "_tests_/root_sizing_tests.rs"]
mod root_sizing_tests;
