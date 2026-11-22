//! The operating system parts of one agent invocation.
//!
//! The generic composition in `application.rs` builds the controller, the
//! daemon, and the CLI actions. It gets each part that depends on the
//! operating system from the native [`Platform`]. Linux gives the systemd,
//! Firecracker, XFS, ext4, and nftables adapters. macOS gives the launchd,
//! Virtualization.framework, APFS, and pf adapters.

use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::boundaries::{
    Cache, DockerDisks, GuestEvent, Host, Images, Service, Store, Updates, Vms,
};
use crate::controller::Agent;
use crate::error::Result;
use crate::preparation::{Preparation, Uninstall};
use crate::runtime_ownership::{RuntimeClaims, RuntimeOwner};
use crate::state::Identity;
use crate::workload_space::WorkloadSpace;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub(crate) use linux::LinuxPlatform as NativePlatform;

/// The platform of this machine and of its job VMs. The agent refuses a
/// runner or an image of another platform.
#[cfg(target_os = "linux")]
pub(crate) const NATIVE: ciproxy_protocol::platform::Platform =
    ciproxy_protocol::platform::Platform::LinuxX64;
#[cfg(target_os = "macos")]
pub(crate) const NATIVE: ciproxy_protocol::platform::Platform =
    ciproxy_protocol::platform::Platform::MacosArm64;
#[cfg(target_os = "macos")]
pub(crate) use macos::MacPlatform as NativePlatform;
#[cfg(target_os = "macos")]
pub(crate) use macos::import_image;
#[cfg(target_os = "macos")]
pub(crate) use macos::prepare_host;
#[cfg(target_os = "macos")]
pub(crate) use macos::vm_helper::run as run_vm_helper;

/// The facts that the machine parts need after the identity loads.
pub(crate) struct MachineInput<'a> {
    /// The command is uninstall, so a data area failure names an uninstall repair.
    pub(crate) uninstall: bool,
    /// The runtime lock of the control directory.
    pub(crate) claims: Arc<dyn RuntimeClaims>,
    /// The runtime claim that the invocation already holds, if any.
    pub(crate) owner: Option<Arc<dyn RuntimeOwner>>,
    /// The saved machine identity, if the machine is registered.
    pub(crate) identity: Option<&'a Identity>,
}

/// The machine parts that the controller uses.
pub(crate) struct Machine {
    /// The bytes that the data area keeps for job images.
    pub(crate) image_budget: u64,
    /// The job image store.
    pub(crate) images: Arc<dyn Images>,
    /// The machine cache service.
    pub(crate) cache: Arc<dyn Cache>,
    /// The VM backend.
    pub(crate) vms: Arc<dyn Vms>,
    /// Guest messages and VM exits from the VM backend.
    pub(crate) guests: mpsc::Receiver<GuestEvent>,
    /// Background work that stops when the invocation drops this value.
    pub(crate) tasks: Box<dyn Any + Send + Sync>,
}

/// The generic parts that the final uninstall uses.
pub(crate) struct UninstallParts {
    /// The runtime claim that the uninstall holds until its cleanup ends.
    pub(crate) runtime: Arc<dyn RuntimeOwner>,
    pub(crate) images: Arc<dyn Images>,
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) service: Arc<dyn Service>,
    pub(crate) store: Arc<dyn Store>,
    pub(crate) agent: Arc<dyn Agent>,
}

/// Build the operating system parts of one agent invocation.
///
/// The generic composition calls the methods in this order: `service`,
/// `host`, `prepare`, `machine`, `dockers`, then the methods of the action.
#[async_trait]
pub(crate) trait Platform: Send + Sync {
    /// Parts from host preparation that the machine parts reuse.
    type Prepared: Send;

    /// The service manager that installs, stops, and removes the daemon.
    fn service(&self) -> Arc<dyn Service>;
    /// The root check, the host name, and the hardware probe.
    fn host(&self) -> Arc<dyn Host>;
    /// Create the control directory, read the host settings, and build host
    /// preparation.
    fn prepare(&self) -> Result<(Arc<dyn Preparation>, Self::Prepared)>;
    /// Build the image store, the machine cache, and the VM backend.
    fn machine(&self, prepared: Self::Prepared, input: MachineInput<'_>) -> Result<Machine>;
    /// Build the Docker disks of job VMs. `None` means that jobs keep Docker
    /// data on the root disk.
    fn dockers(
        &self,
        allocation: Arc<Mutex<()>>,
        space: Arc<dyn WorkloadSpace>,
        image_budget: u64,
    ) -> Option<Arc<dyn DockerDisks>>;
    /// Install or update the machine cache service during install.
    async fn install_cache(&self) -> Result<()>;
    /// Build the hourly agent update.
    fn updates(&self) -> Arc<dyn Updates>;
    /// Build the final uninstall of the machine.
    fn uninstall(&self, parts: UninstallParts) -> Arc<dyn Uninstall>;
}
