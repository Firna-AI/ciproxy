//! One root allocation includes the image, free job bytes, and ext4 overhead.

use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{GIGABYTE, VmSize};

/// Measured logical root length of the current full-image release.
pub const DEFAULT_IMAGE_ROOT_BYTES: u64 = 60_263_759_872;

/// The smallest free job disk accepted for a label.
pub const MINIMUM_JOB_DISK_BYTES: u64 = GIGABYTE;

/// The disk that a macOS guest writes beside the job's own files, such as
/// swap, logs, and the runner's files. A macOS VM counts its label's free
/// job disk plus this margin.
pub const MACOS_ROOT_MARGIN_BYTES: u64 = 4 * GIGABYTE;

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
    /// Use the exact release selected by an image hold or machine report.
    pub fn for_image(image: &ImageMetadata) -> Self {
        Self {
            image_root_bytes: image.root_disk_bytes,
        }
    }

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

/// How the root disk of one own-machine VM takes space on its machine.
///
/// The agent, the scaler, and the control plane use this one rule, so the
/// placement, the runner report, and the dashboard agree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootRule {
    /// Linux: a private ext4 copy of the image root, grown before boot to
    /// the label's free job disk.
    Ext4(RootSizing),
    /// macOS: an APFS clone of the image disk. The clone shares the image's
    /// blocks and takes space only as the job writes. The image budget
    /// already holds the image itself.
    ApfsClone {
        /// The most that one clone can grow, when the image reports it.
        image_free_bytes: Option<u64>,
    },
}

impl RootRule {
    /// The root rule of `platform` for its selected `image`. `fallback`
    /// gives the Linux image root when the machine reports no image.
    pub fn for_platform(
        platform: Platform,
        image: Option<&ImageMetadata>,
        fallback: RootSizing,
    ) -> Self {
        match platform {
            Platform::LinuxX64 => Self::Ext4(image.map_or(fallback, RootSizing::for_image)),
            Platform::MacosArm64 => Self::ApfsClone {
                image_free_bytes: image.and_then(|image| image.free_job_bytes),
            },
        }
    }

    /// The disk that one VM holds for a label's free job disk.
    ///
    /// A macOS VM holds the label's free job disk and
    /// [`MACOS_ROOT_MARGIN_BYTES`]. It gives `None` when the label asks for
    /// more free disk than the image has, because a clone cannot grow past
    /// the fixed image disk.
    pub fn root_bytes(self, free_job_bytes: u64) -> Option<u64> {
        match self {
            Self::Ext4(sizing) => sizing.root_bytes(free_job_bytes),
            Self::ApfsClone { image_free_bytes } => {
                if image_free_bytes.is_some_and(|free| free_job_bytes > free) {
                    return None;
                }
                free_job_bytes.checked_add(MACOS_ROOT_MARGIN_BYTES)
            }
        }
    }

    /// Replace label free bytes with the disk that the VM holds.
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
