//! Check the guest isolation while VMs run, and stop VMs when it is gone.
//!
//! Every 10 seconds the guard loads the pf rules again when they are
//! missing or changed, for example after `pfctl -d` or a flush by another
//! program. It also checks that each guest's DHCP address is inside the
//! subnet that the rules cover. The IPv4 rules match that subnet, so a
//! guest on another vmnet network would escape them. When a check fails,
//! the guard stops the affected VMs, because their isolation is gone. The
//! guest bridge then reports their end to the core.
//!
//! The guard also checks that the area volume is still mounted. When its
//! disk is unplugged, the VM disks are gone, so the guard stops every VM.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_vz::Bundle;
use ciproxy_vz::lease::{LEASE_FILE, lookup};
use tokio::task::JoinHandle;

use super::isolation::Isolation;
use super::launchctl::Launchd;
use crate::macos::area_disk::area_mounted;
use crate::macos::guests::VmFolders;
use crate::macos::pf::VmNetwork;

const INTERVAL: Duration = Duration::from_secs(10);

/// The guard task. Dropping this value stops the guard.
pub(crate) struct IsolationGuard(JoinHandle<()>);

impl Drop for IsolationGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(crate) fn spawn(
    isolation: Arc<dyn Isolation>,
    launchd: Arc<dyn Launchd>,
    folders: VmFolders,
) -> IsolationGuard {
    IsolationGuard(tokio::spawn(async move {
        loop {
            tokio::time::sleep(INTERVAL).await;
            check(isolation.as_ref(), launchd.as_ref(), &folders).await;
        }
    }))
}

async fn check(isolation: &dyn Isolation, launchd: &dyn Launchd, folders: &VmFolders) {
    let running = match launchd.loaded_vms().await {
        Ok(running) => running,
        Err(error) => {
            tracing::warn!(%error, "cannot list VM helpers; the guard retries");
            return;
        }
    };
    if running.is_empty() {
        return;
    }
    if !area_mounted(&folders.area) {
        tracing::error!("the area volume is gone, with the disks of the VMs; stop every VM");
        for local_id in &running {
            if let Err(error) = launchd.bootout_vm(local_id).await {
                tracing::warn!(%error, local_id = %local_id, "the VM stop will retry");
            }
        }
    }
    let network = match isolation.ensure().await {
        Ok(network) => Some(network),
        Err(error) => {
            tracing::error!(%error, "the pf rules are gone; stop every VM");
            None
        }
    };
    for local_id in running {
        if network
            .as_ref()
            .is_some_and(|network| guest_inside(network, folders, &local_id))
        {
            continue;
        }
        tracing::error!(local_id = %local_id, "stop a VM whose isolation is gone");
        if let Err(error) = launchd.bootout_vm(&local_id).await {
            tracing::warn!(%error, local_id = %local_id, "the VM stop will retry");
        }
    }
}

/// A guest without a lease yet has no address to send from, so it passes.
fn guest_inside(network: &VmNetwork, folders: &VmFolders, local_id: &str) -> bool {
    let bundle = Bundle::new(folders.folder(local_id).join("bundle"));
    let Ok(mac) = bundle.read_mac_address() else {
        return true;
    };
    match lookup(LEASE_FILE.as_ref(), mac) {
        Ok(Some(address)) => {
            let inside = network.contains(address);
            if !inside {
                tracing::error!(%address, subnet = %network.subnet(), "a guest is outside the ruled subnet");
            }
            inside
        }
        Ok(None) => true,
        Err(error) => {
            tracing::warn!(%error, "cannot read the DHCP leases; the guard retries");
            true
        }
    }
}
