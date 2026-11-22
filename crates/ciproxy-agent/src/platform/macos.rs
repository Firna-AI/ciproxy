//! macOS parts. Each part returns a "not supported yet" error for now.
//!
//! The macOS agent replaces these stubs with launchd, Virtualization.framework
//! through `ciproxy-vz`, APFS clones, and pf. The root check fails first, so
//! no command starts work on a Mac yet.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::resources::DEFAULT_IMAGE_BUDGET_BYTES;
use tokio::sync::{Mutex, mpsc};

use crate::boundaries::{DockerDisks, Host, Service, Updates};
use crate::error::{Error, Result};
use crate::preparation::{Preparation, Uninstall};
use crate::workload_space::WorkloadSpace;

use super::{Machine, MachineInput, Platform, UninstallParts};

mod unsupported;
pub(crate) mod vm_helper;

use unsupported::{
    UnsupportedCache, UnsupportedHost, UnsupportedImages, UnsupportedPreparation,
    UnsupportedService, UnsupportedUninstall, UnsupportedUpdates, UnsupportedVms,
};

/// The macOS adapters of one invocation.
pub(crate) struct MacPlatform;

impl MacPlatform {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for MacPlatform {
    type Prepared = ();

    fn service(&self) -> Arc<dyn Service> {
        Arc::new(UnsupportedService)
    }

    fn host(&self) -> Arc<dyn Host> {
        Arc::new(UnsupportedHost)
    }

    fn prepare(&self) -> Result<(Arc<dyn Preparation>, ())> {
        Ok((Arc::new(UnsupportedPreparation), ()))
    }

    fn machine(&self, _prepared: (), _input: MachineInput<'_>) -> Result<Machine> {
        let (events, guests) = mpsc::channel(1);
        Ok(Machine {
            image_budget: DEFAULT_IMAGE_BUDGET_BYTES,
            images: Arc::new(UnsupportedImages),
            cache: Arc::new(UnsupportedCache),
            vms: Arc::new(UnsupportedVms { _events: events }),
            guests,
            tasks: Box::new(()),
        })
    }

    fn dockers(
        &self,
        _allocation: Arc<Mutex<()>>,
        _space: Arc<dyn WorkloadSpace>,
        _image_budget: u64,
    ) -> Option<Arc<dyn DockerDisks>> {
        None
    }

    async fn install_cache(&self) -> Result<()> {
        Err(not_supported("the machine cache"))
    }

    fn updates(&self) -> Arc<dyn Updates> {
        Arc::new(UnsupportedUpdates)
    }

    fn uninstall(&self, _parts: UninstallParts) -> Arc<dyn Uninstall> {
        Arc::new(UnsupportedUninstall)
    }
}

/// Name the agent part that macOS does not support yet.
pub(super) fn not_supported(part: &'static str) -> Error {
    Error::PlatformUnsupported { part }
}
