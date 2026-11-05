//! Paths derived only from locally generated identities and verified image keys.

use std::path::PathBuf;

use ciproxy_protocol::identifiers::ImageId;
use sha2::{Digest, Sha256};

use crate::state::VmRecord;

#[derive(Clone)]
pub(crate) struct Layout {
    pub(crate) data: PathBuf,
    pub(crate) firecracker: PathBuf,
    pub(crate) jailer: PathBuf,
}

impl Layout {
    pub(crate) fn jail_root(&self, vm: &VmRecord) -> PathBuf {
        let executable = self.firecracker.file_name().unwrap_or_default();
        self.data
            .join("jailer")
            .join(executable)
            .join(&vm.local_id)
            .join("root")
    }

    pub(crate) fn image(&self, image: &ImageId) -> PathBuf {
        self.data
            .join("images")
            .join(hex::encode(Sha256::digest(image.0.as_bytes())))
    }
}

pub(crate) fn unit_name(vm: &VmRecord) -> String {
    format!("ciproxy-vm-{}.service", vm.local_id)
}

pub(crate) fn jail_uid(vm: &VmRecord) -> u32 {
    100_000 + u32::from(vm.network_index)
}
