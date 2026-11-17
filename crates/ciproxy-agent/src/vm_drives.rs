//! Drive files attached to Firecracker belong to the workload disk allocation.

use std::path::PathBuf;

use crate::layout::Layout;
use crate::state::VmRecord;

/// Every drive file that Firecracker can attach inside one VM jail.
#[derive(Clone, Copy)]
pub(crate) enum VmDrive {
    Root,
    DockerSlot,
    DockerJob,
}

impl VmDrive {
    /// The complete set excluded from image accounting at an actual jail root.
    pub(crate) const ALL: [Self; 3] = [Self::Root, Self::DockerSlot, Self::DockerJob];

    /// The path visible to the jailed Firecracker process.
    pub(crate) fn jailed(self) -> &'static str {
        match self {
            Self::Root => "/rootfs.ext4",
            Self::DockerSlot => "/docker-slot",
            Self::DockerJob => "/docker-job.ext4",
        }
    }

    /// The file name relative to the VM jail root.
    pub(crate) fn file_name(self) -> &'static str {
        self.jailed().trim_start_matches('/')
    }

    /// The host path used to prepare, attach, and remove this drive.
    pub(crate) fn path(self, layout: &Layout, vm: &VmRecord) -> PathBuf {
        layout.jail_root(vm).join(self.file_name())
    }
}
