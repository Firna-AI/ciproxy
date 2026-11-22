//! The macOS VM backend: one launchd VM helper for each job VM.
//!
//! A VM takes the lowest free seat. Its seat gives it a fixed MAC address
//! and machine identifier, so the vmnet DHCP server gives the seat the same
//! lease, and a saved state of the seat can restore.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::runners::RunnerReport;
use ciproxy_vz::{Bundle, Identity, clone_bundle};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::isolation::Isolation;
use super::launchctl::Launchd;
use super::not_supported;
use crate::boundaries::Vms;
use crate::cache_authority::CacheAuthority;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::guests::VmFolders;
use crate::macos::launchd::vm_helper_plist;
use crate::macos::paths::vm_plist;
use crate::macos::seats::{StartKind, free_seat};
use crate::macos::vm_spec::{VM_SPEC_FORMAT, VmSpec};
use crate::persistence::{atomic_private_write, private_directory};
use crate::state::{PreparationState, VmRecord};
use crate::vsock::Guests;

pub(crate) struct MacVms {
    pub(crate) folders: VmFolders,
    pub(crate) launchd: Arc<dyn Launchd>,
    pub(crate) guests: Arc<dyn Guests>,
    pub(crate) isolation: Arc<dyn Isolation>,
    /// Starts and stops run one at a time, as on Linux.
    pub(crate) operation: Mutex<()>,
}

#[async_trait]
impl Vms for MacVms {
    /// Seats are not host resources, so no network outlives its VM.
    async fn networks(&self) -> Result<Vec<u16>> {
        Ok(Vec::new())
    }

    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool> {
        self.guests.configuration_lost(vm).await
    }

    async fn clear_docker(&self, _vm: &VmRecord) -> Result<()> {
        Err(not_supported("Docker disks"))
    }

    async fn swap_docker(&self, _vm: &VmRecord) -> Result<()> {
        Err(not_supported("Docker disks"))
    }

    async fn allocate(&self, report: RunnerReport, used: &[u16]) -> Result<VmRecord> {
        let seat = free_seat(used).ok_or(Error::Capacity)?;
        Ok(VmRecord {
            preparation: PreparationState::Waiting,
            cache_authority: CacheAuthority::Waiting,
            job_facts: None,
            local_id: Uuid::new_v4().simple().to_string(),
            network_index: seat,
            report,
        })
    }

    async fn remove_network(&self, _index: u16) -> Result<()> {
        Ok(())
    }

    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()> {
        self.isolation.ensure().await?;
        {
            let _operation = self.operation.lock().await;
            self.prepare(vm).await?;
            self.launchd.bootstrap_vm(&vm.local_id).await?;
        }
        self.guests.attach(vm, Some(config)).await
    }

    async fn reattach(&self, vm: &VmRecord) -> Result<()> {
        self.guests.attach(vm, None).await
    }

    async fn request_stop(&self, vm: &VmRecord) -> Result<()> {
        self.guests.send(vm, &AgentToGuest::Shutdown).await
    }

    async fn stop(&self, vm: &VmRecord) -> Result<()> {
        let _operation = self.operation.lock().await;
        self.guests.detach(vm).await?;
        self.launchd.bootout_vm(&vm.local_id).await?;
        remove_file(&vm_plist(&vm.local_id))?;
        match std::fs::remove_dir_all(self.folders.folder(&vm.local_id)) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(error).context("remove VM folder")
            }
            _ => Ok(()),
        }
    }

    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        self.guests.send(vm, message).await
    }
}

impl MacVms {
    /// Clone the image with the seat identity, then write `vm.json` and the
    /// helper's launchd file. Every file stays root-only.
    async fn prepare(&self, vm: &VmRecord) -> Result<()> {
        let folder = self.folders.folder(&vm.local_id);
        private_directory(&folder)?;
        let identity = seat_identity(&self.folders.area, vm.network_index)?;
        let image = image_bundle(&self.folders.area, &vm.report.image_id);
        let target = folder.join("bundle");
        tokio::task::spawn_blocking(move || clone_bundle(&image, &target, &identity))
            .await
            .context("join image clone")?
            .map_err(vz)?;
        let spec = VmSpec {
            format: VM_SPEC_FORMAT,
            seat: vm.network_index,
            cpus: vm.report.size.vcpu,
            memory_bytes: vm.report.size.memory_bytes,
            bundle: folder.join("bundle"),
            start: StartKind::Cold,
            saved_state: None,
        };
        atomic_private_write(
            &folder.join("vm.json"),
            &serde_json::to_vec(&spec).context("encode VM file")?,
        )?;
        let plist = vm_plist(&vm.local_id);
        if let Some(parent) = plist.parent() {
            private_directory(parent)?;
        }
        atomic_private_write(&plist, vm_helper_plist(&vm.local_id).as_bytes())
    }
}

/// The checked image bundle of an image. The image store owns this folder.
pub(crate) fn image_bundle(area: &Path, image: &ImageId) -> Bundle {
    Bundle::new(
        area.join("images")
            .join(hex::encode(Sha256::digest(image.0.as_bytes())))
            .join("bundle"),
    )
}

/// The fixed identity of a seat, made once and kept with the seat.
pub(crate) fn seat_identity(area: &Path, seat: u16) -> Result<Identity> {
    let bundle = Bundle::new(seat_identity_folder(area, seat));
    if let Ok(identity) = bundle.read_identity() {
        return Ok(identity);
    }
    let identity = Identity::fresh().map_err(vz)?;
    private_directory(bundle.root())?;
    bundle.write_identity(&identity).map_err(vz)?;
    Ok(identity)
}

fn seat_identity_folder(area: &Path, seat: u16) -> PathBuf {
    area.join("seats").join(seat.to_string()).join("identity")
}

fn remove_file(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove VM launchd file")
        }
        _ => Ok(()),
    }
}

pub(crate) fn vz(error: ciproxy_vz::Error) -> Error {
    Error::VmHelper {
        reason: error.to_string(),
    }
}
