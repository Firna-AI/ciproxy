//! The macOS VM backend: one launchd VM helper for each job VM.
//!
//! A VM takes the lowest free seat. Its seat gives it a fixed MAC address
//! and machine identifier, so the vmnet DHCP server gives the seat the same
//! lease, and a saved state of the seat can restore.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::RunnerReport;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::isolation::Isolation;
use super::launchctl::Launchd;
use super::not_supported;
use super::seat_states::{BuildRequest, SeatStates};
use super::vm_files::{
    RESTORE_FAILED, STATE_FILE, clone_with, image_bundle, remove_helper_files, seat_identity,
    write_helper_files,
};
use crate::boundaries::Vms;
use crate::cache_authority::CacheAuthority;
use crate::error::{Error, Result};
use crate::macos::area_disk::{MISSING_REPAIR, area_mounted};
use crate::macos::guests::{
    HELPER_EXIT_WAIT, HELPER_START_WAIT, VmFolders, wait_for_helper, wait_for_helper_end,
};
use crate::macos::seats::{SeatNeeds, StartKind, free_seat};
use crate::macos::vm_spec::{VM_SPEC_FORMAT, VmSpec};
use crate::persistence::private_directory;
use crate::state::{PreparationState, VmRecord};
use crate::vsock::Guests;

pub(crate) struct MacVms {
    pub(crate) folders: VmFolders,
    pub(crate) launchd: Arc<dyn Launchd>,
    pub(crate) guests: Arc<dyn Guests>,
    pub(crate) isolation: Arc<dyn Isolation>,
    pub(crate) seats: Arc<SeatStates>,
    /// The daemon builds saved states. Install and uninstall do not.
    pub(crate) builds: bool,
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
            self.seats.cancel(vm.network_index).await?;
            self.prepare(vm).await?;
            self.launchd.bootstrap_vm(&vm.local_id).await?;
        }
        wait_for_helper(&self.folders, &vm.local_id, HELPER_START_WAIT).await?;
        self.guests.attach(vm, Some(config)).await
    }

    async fn reattach(&self, vm: &VmRecord) -> Result<()> {
        self.guests.attach(vm, None).await
    }

    async fn request_stop(&self, vm: &VmRecord) -> Result<()> {
        self.guests.send(vm, &AgentToGuest::Shutdown).await
    }

    async fn stop(&self, vm: &VmRecord) -> Result<()> {
        {
            let _operation = self.operation.lock().await;
            self.guests.detach(vm).await?;
            self.launchd.bootout_vm(&vm.local_id).await?;
            wait_for_helper_end(&self.folders, &vm.local_id, HELPER_EXIT_WAIT).await?;
            let restore_failed = self.folders.folder(&vm.local_id).join(RESTORE_FAILED);
            if restore_failed.exists() {
                self.seats.discard(vm.network_index)?;
            }
            remove_helper_files(&self.folders, &vm.local_id)?;
        }
        if self.builds {
            let request = BuildRequest {
                seat: vm.network_index,
                image: vm.report.image_id.clone(),
                cpus: vm.report.size.vcpu,
                memory_bytes: vm.report.size.memory_bytes,
            };
            let identity = seat_identity(&self.folders.area, request.seat)?;
            let mac_address = identity.mac_address.to_string();
            if self.seats.usable(needs(vm, &mac_address)).is_none() {
                self.seats.start_build(request).await;
            }
        }
        Ok(())
    }

    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        self.guests.send(vm, message).await
    }
}

impl MacVms {
    /// Clone the seat's saved state when it matches the VM, else the image,
    /// with the seat identity. Then write `vm.json` and the launchd file.
    async fn prepare(&self, vm: &VmRecord) -> Result<()> {
        if !area_mounted(&self.folders.area) {
            return Err(Error::Preparation {
                component: "area volume",
                repair: MISSING_REPAIR,
            });
        }
        let folder = self.folders.folder(&vm.local_id);
        private_directory(&folder)?;
        let identity = seat_identity(&self.folders.area, vm.network_index)?;
        let mac_address = identity.mac_address.to_string();
        let bundle = folder.join("bundle");
        let (source, start) = match self.seats.usable(needs(vm, &mac_address)) {
            Some(saved) => (saved, StartKind::Restore),
            None => (
                image_bundle(&self.folders.area, &vm.report.image_id),
                StartKind::Cold,
            ),
        };
        clone_with(source, bundle.clone(), identity).await?;
        let spec = VmSpec {
            format: VM_SPEC_FORMAT,
            seat: vm.network_index,
            cpus: vm.report.size.vcpu,
            memory_bytes: vm.report.size.memory_bytes,
            saved_state: (start == StartKind::Restore).then(|| bundle.join(STATE_FILE)),
            bundle,
            start,
        };
        tracing::info!(seat = vm.network_index, start = ?spec.start, "prepare a job VM");
        write_helper_files(&self.folders, &vm.local_id, &spec)
    }
}

fn needs<'a>(vm: &'a VmRecord, mac_address: &'a str) -> SeatNeeds<'a> {
    SeatNeeds {
        seat: vm.network_index,
        newest_image: &vm.report.image_id.0,
        mac_address,
        cpus: vm.report.size.vcpu,
        memory_bytes: vm.report.size.memory_bytes,
        agent_version: env!("CARGO_PKG_VERSION"),
        now_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |time| time.as_secs()),
    }
}
